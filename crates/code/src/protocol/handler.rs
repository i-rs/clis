use crate::agent::Agent;
use crate::protocol::{CodeEvent, ClawTask, transport::Transport};

pub async fn run_agent_loop(agent: &mut Agent, task_id: &str) -> anyhow::Result<()> {
    // Set agent mode flag so tools know they're running under claw
    unsafe { std::env::set_var("I_RS_CODE_AGENT_MODE", "1"); }

    // Notify claw we're ready
    Transport::send_event(&CodeEvent::progress(task_id, "ready", "i-rs-code agent ready"))?;

    // Read task from stdin (sent by claw)
    let line = Transport::read_line().await?;
    let task: ClawTask = serde_json::from_str(&line)?;

    if task.msg_type != "task" {
        anyhow::bail!("Expected task message, got: {}", task.msg_type);
    }

    let prompt = task.prompt.unwrap_or_default();
    Transport::send_event(&CodeEvent::progress(task_id, "processing", &format!("Starting task: {}", &prompt[..prompt.len().min(80)])))?;

    // Run agent, intercepting claw requests
    agent.add_system_prompt("You are i-rs-code running under claw supervision. When you need help (build errors, design review, user approval), use the call_claw tool. After creating a tool, use register_tool to register it. Always verify your work with cargo check.");

    // Build messages and run with claw interaction support
    let tool_defs = agent.tools.schemas();

    let mut messages = Vec::new();
    messages.push(crate::provider::LlmMessage::System(
        "You are i-rs-code running under claw supervision. When you need help (build errors, design review, user approval), use the call_claw tool. After creating a tool, use register_tool to register it. Always verify your work with cargo check.".into()
    ));
    messages.push(crate::provider::LlmMessage::User(prompt.clone()));

    loop {
        let (final_text, new_messages) = crate::agent::engine::react_loop(
            &*agent.provider,
            &agent.tools,
            messages,
            &tool_defs,
            true, // json_output always true in agent mode
        ).await?;

        messages = new_messages;

        // Check if the react loop ended with a claw request
        if let Some(last) = messages.last() {
            if let crate::provider::LlmMessage::Tool { content, .. } = last {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
                    if val.get("requires_claw").and_then(|v| v.as_bool()).unwrap_or(false) {
                        // Send request to claw, wait for response
                        let request_type = val.get("request_type").and_then(|v| v.as_str()).unwrap_or("info");
                        let content = val.get("content").and_then(|v| v.as_str()).unwrap_or("");

                        Transport::send_event(&CodeEvent::request(
                            task_id, "req-1", request_type, content, None,
                        ))?;

                        // Wait for claw's respond
                        let respond_line = Transport::read_line().await?;
                        let respond: ClawTask = serde_json::from_str(&respond_line)?;

                        if respond.msg_type == "respond" {
                            let response = respond.content.unwrap_or_default();
                            messages.push(crate::provider::LlmMessage::User(
                                format!("[Claw's response to your request]: {}", response)
                            ));
                            continue; // Continue ReAct loop with the response
                        }
                        break;
                    }

                    if val.get("requires_registration").and_then(|v| v.as_bool()).unwrap_or(false) {
                        if let Some(tool_info) = val.get("tool") {
                            Transport::send_event(&CodeEvent::tool_created(task_id, tool_info.clone()))?;
                        }
                    }
                }
            }
        }

        // If we got here, the react loop finished normally
        if !final_text.is_empty() {
            Transport::send_event(&CodeEvent::done(task_id, &final_text))?;
        } else {
            Transport::send_event(&CodeEvent::done(task_id, "Task completed"))?;
        }
        break;
    }

    Ok(())
}
