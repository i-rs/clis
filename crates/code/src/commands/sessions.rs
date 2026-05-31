use super::helpers::{print_content, truncate};
use crate::config;

pub async fn run_sessions_list() -> anyhow::Result<()> {
    let sessions_dir = config::i_rs_code_dir().join("sessions");
    if !sessions_dir.exists() {
        println!("No sessions found.");
        return Ok(());
    }

    let mut entries: Vec<(String, String, usize, String)> = Vec::new();
    for entry in std::fs::read_dir(&sessions_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let session: crate::session::Session = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => continue,
        };
        entries.push((
            session.id,
            session.created_at,
            session.messages.len(),
            session.updated_at,
        ));
    }

    entries.sort_by(|a, b| b.1.cmp(&a.1));

    if entries.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }

    println!("Sessions ({} total):", entries.len());
    println!();
    for (id, created, count, updated) in &entries {
        let short_id = if id.len() > 8 { &id[..8] } else { id.as_str() };
        println!(
            "  {:<12} │ {} msgs │ created: {} │ updated: {}",
            short_id, count, created, updated
        );
    }
    println!();
    println!("Use `i-rs-code sessions show <id>` to view details.");
    Ok(())
}

pub async fn run_sessions_show(id: &str, full: bool) -> anyhow::Result<()> {
    let sessions_dir = config::i_rs_code_dir().join("sessions");
    let session = crate::session::Session::load(id, &sessions_dir)?;

    println!("Session: {}", session.id);
    println!("Created: {}", session.created_at);
    println!("Updated: {}", session.updated_at);
    println!("Messages: {}", session.messages.len());
    println!();

    for (i, msg) in session.messages.iter().enumerate() {
        match msg {
            crate::app::AgentMessage::User { content } => {
                println!("── [{}. User] ──", i + 1);
                print_content(content, full);
            }
            crate::app::AgentMessage::Assistant {
                content, reasoning, ..
            } => {
                println!("── [{}. Assistant] ──", i + 1);
                if !reasoning.is_empty() {
                    println!("  [reasoning]: {}", truncate(reasoning, 200));
                }
                print_content(content, full);
            }
            crate::app::AgentMessage::ToolResult { content, .. } => {
                println!("── [{}. Tool Result] ──", i + 1);
                let preview: String = content.chars().take(300).collect();
                if content.len() > 300 {
                    println!("  {}...", preview);
                } else {
                    println!("  {}", preview);
                }
            }
            crate::app::AgentMessage::System { content } => {
                println!("── [{}. System] ──", i + 1);
                print_content(content, full);
            }
            crate::app::AgentMessage::FileEdit { path, summary } => {
                println!("── [{}. File Edit] ──", i + 1);
                println!("  {}: {}", path, summary);
            }
            crate::app::AgentMessage::Separator { label } => {
                println!("  ── {} ──", label);
            }
        }
    }
    Ok(())
}

pub async fn run_sessions_delete(id: &str) -> anyhow::Result<()> {
    let sessions_dir = config::i_rs_code_dir().join("sessions");
    let path = sessions_dir.join(format!("{}.json", id));
    if !path.exists() {
        anyhow::bail!("Session '{}' not found at {:?}", id, path);
    }
    std::fs::remove_file(&path)?;
    println!("✓ Session '{}' deleted.", id);
    Ok(())
}

pub async fn run_sessions_export(id: &str) -> anyhow::Result<()> {
    let sessions_dir = config::i_rs_code_dir().join("sessions");
    let session = crate::session::Session::load(id, &sessions_dir)?;

    println!("# Session: {}", session.id);
    println!();
    println!("- **Created**: {}", session.created_at);
    println!("- **Updated**: {}", session.updated_at);
    println!("- **Messages**: {}", session.messages.len());
    println!();

    for msg in &session.messages {
        match msg {
            crate::app::AgentMessage::User { content } => {
                println!("## User\n");
                println!("{}", content);
                println!();
            }
            crate::app::AgentMessage::Assistant {
                content, reasoning, ..
            } => {
                println!("## Assistant\n");
                if !reasoning.is_empty() {
                    println!("> **Reasoning**: {}\n", reasoning);
                }
                println!("{}", content);
                println!();
            }
            crate::app::AgentMessage::ToolResult { content, .. } => {
                println!("### Tool Result\n");
                println!("```");
                println!("{}", content);
                println!("```");
                println!();
            }
            crate::app::AgentMessage::System { content } => {
                println!("### System\n");
                println!("```");
                println!("{}", content);
                println!("```");
                println!();
            }
            crate::app::AgentMessage::FileEdit { path, summary } => {
                println!("### File Edit: `{}`", path);
                println!("{}", summary);
                println!();
            }
            crate::app::AgentMessage::Separator { label } => {
                println!("--- *{}* ---", label);
                println!();
            }
        }
    }
    Ok(())
}
