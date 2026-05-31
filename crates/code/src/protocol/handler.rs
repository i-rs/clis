use crate::agent::Agent;
use crate::protocol::{
    ClawTask, CodeEvent,
    transport::{self},
};
use std::sync::atomic::{AtomicBool, Ordering};

pub static AGENT_MODE: AtomicBool = AtomicBool::new(false);

pub async fn run_agent_loop(agent: &mut Agent, task_id: &str) -> anyhow::Result<()> {
    AGENT_MODE.store(true, Ordering::SeqCst);

    // Notify claw we're ready
    transport::send_event(&CodeEvent::progress(
        task_id,
        "ready",
        "i-rs-code agent ready",
    ))?;

    // Read task from stdin (sent by claw)
    let line = transport::read_line().await?;
    let task: ClawTask = serde_json::from_str(&line)?;

    if task.msg_type != "task" {
        anyhow::bail!("Expected task message, got: {}", task.msg_type);
    }

    let prompt = task.prompt.unwrap_or_default();
    transport::send_event(&CodeEvent::progress(
        task_id,
        "processing",
        &format!("Starting task: {}", &prompt[..prompt.len().min(80)]),
    ))?;

    // Run agent, intercepting claw requests
    agent.add_system_prompt("You are i-rs-code running under claw supervision. When you need help (build errors, design review, user approval), use the call_claw tool. After creating a tool, use register_tool to register it. Always verify your work with cargo check.");

    // Build messages and run with claw interaction support
    let tool_defs = agent.tools.schemas();

    let mut messages = vec![
        crate::provider::LlmMessage::System(
            "You are i-rs-code running under claw supervision. When you need help (build errors, design review, user approval), use the call_claw tool. After creating a tool, use register_tool to register it. Always verify your work with cargo check.".into()
        ),
        crate::provider::LlmMessage::User(prompt.clone()),
    ];

    loop {
        let (final_text, new_messages) = crate::agent::engine::react_loop(
            &*agent.provider,
            &agent.tools,
            messages,
            &tool_defs,
            true,
            agent.config.max_rounds,
            agent.config.tool_timeout_secs,
            &mut agent.memory,
        )
        .await?;

        messages = new_messages;

        // Check if the react loop ended with a claw request
        if let Some(crate::provider::LlmMessage::Tool { content, .. }) = messages.last() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(content)
                && val
                    .get("requires_claw")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            {
                let request_type = val
                    .get("request_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("info");
                let content = val.get("content").and_then(|v| v.as_str()).unwrap_or("");

                transport::send_event(&CodeEvent::request(
                    task_id,
                    "req-1",
                    request_type,
                    content,
                    None,
                ))?;

                let respond_line = transport::read_line().await?;
                let respond: ClawTask = serde_json::from_str(&respond_line)?;

                if respond.msg_type == "respond" {
                    let response = respond.content.unwrap_or_default();
                    messages.push(crate::provider::LlmMessage::User(format!(
                        "[Claw's response to your request]: {}",
                        response
                    )));
                    continue;
                }
                break;
            }

            if let Ok(val) = serde_json::from_str::<serde_json::Value>(content)
                && val
                    .get("requires_registration")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                && let Some(tool_info) = val.get("tool")
            {
                transport::send_event(&CodeEvent::tool_created(task_id, tool_info.clone()))?;
            }
        }

        // If we got here, the react loop finished normally
        if !final_text.is_empty() {
            transport::send_event(&CodeEvent::done(task_id, &final_text))?;
        } else {
            transport::send_event(&CodeEvent::done(task_id, "Task completed"))?;
        }
        break;
    }

    Ok(())
}
