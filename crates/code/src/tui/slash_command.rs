use crate::app::{AgentMessage, App};
use crate::config;

/// Parsed slash command variants
#[derive(Debug)]
pub enum SlashCommand {
    // Tier 1 — pure in-memory
    Clear,
    Help,
    Tools,
    Status,
    New,
    Undo,
    // Tier 2 — config / runtime state
    Model(String),
    Temperature(f64),
    Retry,
    // Tier 3 — persistence / I/O
    Save(String),
    Load(String),
    Sessions,
    ExportMd,
    Files,
}

/// Help entry for a slash command
pub struct CmdHelp {
    pub name: &'static str,
    pub args: &'static str,
    pub desc: &'static str,
    pub category: &'static str,
}

pub static COMMANDS: &[CmdHelp] = &[
    CmdHelp { name: "clear", args: "", desc: "清空当前对话", category: "对话管理" },
    CmdHelp { name: "new", args: "", desc: "重置为全新会话", category: "对话管理" },
    CmdHelp { name: "undo", args: "", desc: "撤回上一条对话", category: "对话管理" },
    CmdHelp { name: "retry", args: "", desc: "移除上一条回复，可重新发送", category: "对话管理" },
    CmdHelp { name: "help", args: "", desc: "显示所有可用命令", category: "帮助" },
    CmdHelp { name: "status", args: "", desc: "显示当前状态", category: "信息" },
    CmdHelp { name: "tools", args: "", desc: "列出可用工具", category: "信息" },
    CmdHelp { name: "model", args: "<name>", desc: "切换 LLM 模型", category: "配置" },
    CmdHelp { name: "temp", args: "<n>", desc: "设置 temperature（0.0–2.0）", category: "配置" },
    CmdHelp { name: "save", args: "<name>", desc: "保存当前会话", category: "会话" },
    CmdHelp { name: "load", args: "<name>", desc: "加载已保存会话", category: "会话" },
    CmdHelp { name: "sessions", args: "", desc: "列出所有保存的会话", category: "会话" },
    CmdHelp { name: "export", args: "md", desc: "导出对话为 Markdown 文件", category: "导出" },
    CmdHelp { name: "files", args: "", desc: "列出当前工作区文件", category: "工作区" },
];

/// Parse a raw input line into a `SlashCommand`.
/// Returns `Err` for unknown commands or invalid arguments.
pub fn parse(input: &str) -> Result<SlashCommand, String> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return Err("not a slash command".into());
    }
    let rest = trimmed[1..].trim();
    let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
    let cmd = parts[0].to_lowercase();
    let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

    match cmd.as_str() {
        "clear" => Ok(SlashCommand::Clear),
        "help" | "?" => Ok(SlashCommand::Help),
        "tools" => Ok(SlashCommand::Tools),
        "status" | "st" => Ok(SlashCommand::Status),
        "new" => Ok(SlashCommand::New),
        "undo" => Ok(SlashCommand::Undo),
        "model" => {
            if arg.is_empty() {
                return Err("Usage: /model <name>  —  e.g. /model gpt-4o".into());
            }
            Ok(SlashCommand::Model(arg.to_string()))
        }
        "temp" | "temperature" => {
            let temp: f64 = arg
                .parse()
                .map_err(|_| "Usage: /temp <n>  —  e.g. /temp 0.7".to_string())?;
            if !(0.0..=2.0).contains(&temp) {
                return Err("Temperature must be between 0.0 and 2.0".into());
            }
            Ok(SlashCommand::Temperature(temp))
        }
        "retry" => Ok(SlashCommand::Retry),
        "save" => {
            if arg.is_empty() {
                return Err("Usage: /save <name>  —  e.g. /save my-work".into());
            }
            Ok(SlashCommand::Save(arg.to_string()))
        }
        "load" => {
            if arg.is_empty() {
                return Err("Usage: /load <name>  —  use /sessions to list available".into());
            }
            Ok(SlashCommand::Load(arg.to_string()))
        }
        "sessions" | "ls" => Ok(SlashCommand::Sessions),
        "export" => {
            if arg == "md" || arg == "markdown" {
                Ok(SlashCommand::ExportMd)
            } else {
                Err("Usage: /export md  —  export conversation as Markdown".into())
            }
        }
        "files" => Ok(SlashCommand::Files),
        _ => Err(format!(
            "Unknown command: /{}. Type /help to see available commands.",
            cmd
        )),
    }
}

/// Execute a parsed slash command, mutating `app` and returning system messages to display.
pub async fn execute(cmd: SlashCommand, app: &mut App) -> Vec<AgentMessage> {
    match cmd {
        // ── Tier 1 ──────────────────────────────────────────
        SlashCommand::Clear => cmd_clear(app),
        SlashCommand::Help => cmd_help(),
        SlashCommand::Tools => cmd_tools(app),
        SlashCommand::Status => cmd_status(app),
        SlashCommand::New => cmd_new(app),
        SlashCommand::Undo => cmd_undo(app),

        // ── Tier 2 ──────────────────────────────────────────
        SlashCommand::Model(name) => cmd_model(app, &name),
        SlashCommand::Temperature(temp) => cmd_temperature(app, temp),
        SlashCommand::Retry => cmd_retry(app),

        // ── Tier 3 ──────────────────────────────────────────
        SlashCommand::Save(name) => cmd_save(app, &name).await,
        SlashCommand::Load(name) => cmd_load(app, &name).await,
        SlashCommand::Sessions => cmd_sessions().await,
        SlashCommand::ExportMd => cmd_export_md(app).await,
        SlashCommand::Files => cmd_files(app).await,
    }
}

// ============================================================
//  Command handlers
// ============================================================

fn cmd_clear(app: &mut App) -> Vec<AgentMessage> {
    app.messages.clear();
    app.agent_messages.clear();
    app.streaming = None;
    app.token_usage = crate::app::TokenUsage::default();
    app.file_changes.clear();
    app.last_file_states.clear();
    app.status_message = None;
    // Preserve tool_names so tab completion still works
    vec![AgentMessage::system("对话已清空。")]
}

fn cmd_help() -> Vec<AgentMessage> {
    let mut lines = vec![
        "── Slash 命令 ──".to_string(),
        String::new(),
    ];
    let mut current_cat = String::new();
    for cmd in COMMANDS {
        if cmd.category != current_cat {
            current_cat = cmd.category.to_string();
            lines.push(format!("  {}:", current_cat));
        }
        let args = if cmd.args.is_empty() {
            String::new()
        } else {
            format!(" {}", cmd.args)
        };
        lines.push(format!("    /{}{}  —  {}", cmd.name, args, cmd.desc));
    }
    lines.push(String::new());
    lines.push("提示：输入命令后按 Enter 执行。Tab 键可补全命令名。".to_string());
    vec![AgentMessage::system(lines.join("\n"))]
}

fn cmd_tools(app: &App) -> Vec<AgentMessage> {
    if app.tool_names.is_empty() {
        return vec![AgentMessage::system("没有可用工具。")];
    }
    let mut lines = vec![format!("可用工具（{} 个）:", app.tool_names.len())];
    for name in &app.tool_names {
        lines.push(format!("  {}", name));
    }
    vec![AgentMessage::system(lines.join("\n"))]
}

fn cmd_status(app: &App) -> Vec<AgentMessage> {
    let model = app.config.effective_model();
    let provider = &app.config.provider;
    let msg_count = app.messages.len();
    let tool_count = app.messages.iter().filter(|m| matches!(m, AgentMessage::ToolResult { .. })).count();
    let input_tokens = app.token_usage.input;
    let output_tokens = app.token_usage.output;
    let temperature = app.temperature;
    let context_pct = app.context_usage.map(|p| format!("{:.0}%", p * 100.0)).unwrap_or_else(|| "—".into());

    vec![AgentMessage::system(format!(
        "── 状态 ──\n\
         模型:      {} ({})\n\
         温度:      {:.1}\n\
         消息:      {}条（{}次工具调用）\n\
         Token:     {} in / {} out\n\
         上下文:    {}",
        model, provider, temperature, msg_count, tool_count, input_tokens, output_tokens, context_pct,
    ))]
}

fn cmd_new(app: &mut App) -> Vec<AgentMessage> {
    app.messages.clear();
    app.agent_messages.clear();
    app.streaming = None;
    app.token_usage = crate::app::TokenUsage::default();
    app.file_changes.clear();
    app.last_file_states.clear();
    app.status_message = None;
    app.context_usage = None;
    app.session_id = None;
    app.messages.push(AgentMessage::Assistant {
        content: format!(
            "已开始新会话。\n\n\
             Type a message to start coding...\n\n\
             Available commands:\n  \
             i-rs-code chat <prompt>  One-shot conversation\n  \
             i-rs-code config init    Interactive setup\n  \
             i-rs-code config show    View configuration"
        ),
        reasoning: String::new(),
        tool_calls: None,
        reasoning_expanded: false,
    });
    vec![]
}

fn cmd_undo(app: &mut App) -> Vec<AgentMessage> {
    // Walk backwards to find the last User message, then truncate before it
    let mut remove_idx = None;
    for (i, msg) in app.messages.iter().enumerate().rev() {
        if matches!(msg, AgentMessage::User { .. }) {
            remove_idx = Some(i);
            break;
        }
    }
    match remove_idx {
        Some(idx) => {
            app.messages.truncate(idx);
            vec![AgentMessage::system("已撤回最后一条对话。")]
        }
        None => vec![AgentMessage::system("没有可撤回的对话。")],
    }
}

// ── Tier 2 ──────────────────────────────────────────

fn cmd_model(app: &mut App, name: &str) -> Vec<AgentMessage> {
    let old = app.config.effective_model().to_string();
    app.config.model = Some(name.to_string());
    vec![AgentMessage::system(format!(
        "模型已切换: {} → {}",
        old, name
    ))]
}

fn cmd_temperature(app: &mut App, temp: f64) -> Vec<AgentMessage> {
    let old = app.temperature;
    app.temperature = temp;
    vec![AgentMessage::system(format!(
        "Temperature 已设置: {:.1} → {:.1}",
        old, temp
    ))]
}

fn cmd_retry(app: &mut App) -> Vec<AgentMessage> {
    // Walk backwards, find the last Assistant message, truncate there
    // keeping the User message and everything before it intact
    let mut remove_idx = None;
    for (i, msg) in app.messages.iter().enumerate().rev() {
        if matches!(msg, AgentMessage::Assistant { .. }) {
            remove_idx = Some(i);
            break;
        }
    }
    match remove_idx {
        Some(idx) => {
            app.messages.truncate(idx);
            // Also roll back agent_messages to match
            app.agent_messages.retain(|m| {
                matches!(m, crate::provider::LlmMessage::System(_) | crate::provider::LlmMessage::User(_))
            });
            vec![AgentMessage::system(
                "已移除上一条回复。修改输入后按 Enter 重新发送，或直接按 ↑ 调出上一条输入。",
            )]
        }
        None => vec![AgentMessage::system("没有可重试的回复。")],
    }
}

// ── Tier 3 ──────────────────────────────────────────

async fn cmd_save(app: &mut App, name: &str) -> Vec<AgentMessage> {
    let sessions_dir = config::i_rs_code_dir().join("sessions");
    let session = crate::session::Session::from_agent_messages(
        Some(name.to_string()),
        &app.messages,
        app.agent_messages.clone(),
    );
    match session.save(&sessions_dir) {
        Ok(_) => vec![AgentMessage::system(format!("会话已保存: {}", name))],
        Err(e) => vec![AgentMessage::system(format!("保存失败: {}", e))],
    }
}

async fn cmd_load(app: &mut App, name: &str) -> Vec<AgentMessage> {
    let sessions_dir = config::i_rs_code_dir().join("sessions");
    match crate::session::Session::load(name, &sessions_dir) {
        Ok(s) => {
            app.messages = s.messages;
            app.agent_messages = s.agent_messages;
            app.streaming = None;
            app.token_usage = crate::app::TokenUsage::default();
            app.session_id = Some(name.to_string());
            vec![AgentMessage::system(format!("已加载会话: {}", name))]
        }
        Err(e) => vec![AgentMessage::system(format!("加载失败: {}", e))],
    }
}

async fn cmd_sessions() -> Vec<AgentMessage> {
    let sessions_dir = config::i_rs_code_dir().join("sessions");
    if !sessions_dir.exists() {
        return vec![AgentMessage::system("没有保存的会话。")];
    }

    let mut ids: Vec<String> = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&sessions_dir).await {
        loop {
            match entries.next_entry().await {
                Ok(Some(entry)) => {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("json") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            ids.push(stem.to_string());
                        }
                    }
                }
                Ok(None) => break,
                Err(_) => continue,
            }
        }
    }
    ids.sort();

    if ids.is_empty() {
        return vec![AgentMessage::system("没有保存的会话。")];
    }

    let mut lines = vec![format!("已保存的会话（{} 个）:", ids.len())];
    for id in &ids {
        let short = if id.len() > 12 { &id[..12] } else { id.as_str() };
        lines.push(format!("  /load {}", short));
    }
    lines.push(String::new());
    lines.push("使用 /load <name> 加载对应会话。".to_string());
    vec![AgentMessage::system(lines.join("\n"))]
}

async fn cmd_export_md(app: &App) -> Vec<AgentMessage> {
    let mut md = String::new();
    md.push_str(&format!("# i-rs-code 对话 - {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    md.push_str(&format!("- **模型**: {}\n", app.config.effective_model()));
    md.push_str(&format!("- **消息数**: {}\n", app.messages.len()));
    md.push_str(&format!("- **Token 用量**: {} in / {} out\n\n", app.token_usage.input, app.token_usage.output));
    md.push_str("---\n\n");

    for msg in &app.messages {
        match msg {
            AgentMessage::User { content } => {
                md.push_str("## User\n\n");
                md.push_str(content);
                md.push_str("\n\n");
            }
            AgentMessage::Assistant { content, reasoning, .. } => {
                md.push_str("## Assistant\n\n");
                if !reasoning.is_empty() {
                    md.push_str("> **思考过程**:\n>\n");
                    for line in reasoning.lines() {
                        md.push_str(&format!("> {}\n", line));
                    }
                    md.push_str("\n");
                }
                md.push_str(content);
                md.push_str("\n\n");
            }
            AgentMessage::ToolResult { content } => {
                md.push_str("### Tool Result\n\n```\n");
                md.push_str(content);
                md.push_str("\n```\n\n");
            }
            AgentMessage::System { content } => {
                md.push_str("### System\n\n```\n");
                md.push_str(content);
                md.push_str("\n```\n\n");
            }
            AgentMessage::FileEdit { path, summary } => {
                md.push_str(&format!("### File Edit: `{}`\n\n{}\n\n", path, summary));
            }
            AgentMessage::Separator { .. } => {
                md.push_str("---\n\n");
            }
        }
    }

    let filename = format!(
        "i-rs-code-export-{}.md",
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    );
    match tokio::fs::write(&filename, &md).await {
        Ok(_) => vec![AgentMessage::system(format!("对话已导出: {}", filename))],
        Err(e) => vec![AgentMessage::system(format!("导出失败: {}", e))],
    }
}

async fn cmd_files(app: &App) -> Vec<AgentMessage> {
    let dir = &app.current_dir;
    let mut files: Vec<String> = Vec::new();
    let mut dirs: Vec<String> = Vec::new();
    let mut count = 0u32;

    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(d) => d,
        Err(e) => return vec![AgentMessage::system(format!("读取目录失败: {}", e))],
    };

    loop {
        if count >= 60 {
            files.push("... (超过 60 项，已截断)".to_string());
            break;
        }
        let entry = match entries.next_entry().await {
            Ok(Some(e)) => e,
            Ok(None) => break,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        count += 1;
        if entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
            dirs.push(format!("  📁 {}", name));
        } else {
            let size = entry.metadata().await.map(|m| m.len()).unwrap_or(0);
            let size_str = if size > 1024 * 1024 {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            } else if size > 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{} B", size)
            };
            files.push(format!("  📄 {}  ({})", name, size_str));
        }
    }

    let mut lines = Vec::new();
    lines.push(format!("工作区: {}", dir));
    lines.push(String::new());
    lines.push(format!("目录（{} 个）:", dirs.len()));
    lines.extend(dirs);
    lines.push(String::new());
    lines.push(format!("文件（{} 个）:", files.len()));
    lines.extend(files);

    vec![AgentMessage::system(lines.join("\n"))]
}
