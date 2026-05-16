mod app;
mod config;
mod llm;
mod memory;
mod session;
mod tool_cache;
mod tools;
mod ui;
mod utils;

use crate::config::Config;
use crate::llm::LlmEvent;
use crate::session::SessionManager;
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tokio::sync::mpsc;

#[derive(Parser)]
#[command(name = "i-rs-claw", version, about = "TUI intelligent personal data assistant for i-rs CLI tools")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Launch the TUI assistant (default)
    Tui,
    /// Show configuration
    Config,
    /// List and manage sessions
    Session {
        /// List all sessions
        #[arg(long)]
        list: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Tui) {
        Command::Tui => run_tui(),
        Command::Config => run_config(),
        Command::Session { list: true } => run_session_list(),
        Command::Session { list: false } => run_session_list(),
    }
}

// =============================================
// Config subcommand
// =============================================

fn run_config() -> anyhow::Result<()> {
    let claw_dir = claw_dir();
    let config_path = claw_dir.join("config.toml");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        println!("配置文件: {}", config_path.display());
        println!("{}", content);
    } else {
        eprintln!("配置文件不存在: {}", config_path.display());
        eprintln!("\n请创建该文件，示例：");
        eprintln!("[config]");
        eprintln!("api_key = \"sk-...\"");
        eprintln!("# base_url = \"https://api.openai.com/v1\"");
        eprintln!("# model = \"gpt-4o-mini\"");
    }
    Ok(())
}

// =============================================
// Session subcommand
// =============================================

fn run_session_list() -> anyhow::Result<()> {
    let session_mgr = SessionManager::new(claw_dir().join("claw"));

    let sessions = session_mgr.sessions();
    if sessions.is_empty() {
        println!("暂无会话");
        return Ok(());
    }

    println!("会话列表 ({} 个):\n", sessions.len());
    for (i, s) in sessions.iter().enumerate() {
        let time = chrono::DateTime::from_timestamp(s.created_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();
        println!("  {}. {}", i + 1, s.title);
        println!("     {} 条消息 | {}", s.message_count, time);
    }
    Ok(())
}

// =============================================
// TUI subcommand (original behavior)
// =============================================

fn run_tui() -> anyhow::Result<()> {
    let config = Config::load()?;

    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let rt = tokio::runtime::Runtime::new()?;
    let (llm_tx, mut llm_rx) = mpsc::unbounded_channel::<LlmEvent>();

    let mut app = app::App::new(config);

    // Initialize tool doc cache, session manager, and cross-session memory
    let claw_dir = claw_dir().join("claw");
    let tool_cache = tool_cache::ToolDocCache::new(claw_dir.clone());
    let mut session_mgr = session::SessionManager::new(claw_dir.clone());
    let mut cross_memory = memory::CrossSessionMemory::new(claw_dir);

    // Ensure at least one session exists
    if session_mgr.current_id().is_none() {
        session_mgr.create_session();
    }

    // Analyze cross-session tool usage from all sessions
    cross_memory.analyze_sessions(session_mgr.sessions(), &session_mgr);

    // Load messages from current session
    let session_id = session_mgr.current_id().unwrap().to_string();
    let loaded = session_mgr.load_app_messages(&session_id, 50);
    app.messages = loaded;

    // If first-time user (no saved messages), show welcome
    if app.messages.is_empty() {
        app.messages.push(app::Message::Assistant {
            text: "你好！我是 i-rs-claw，你的个人数据智能助理。\
                   \n我可以帮你管理健康、财务、任务、媒体等个人信息。\
                   \n试试说：\"记录体重75kg\" 或 \"最近跑步情况如何？\""
                .to_string(),
        });
    }

    let result = tui_main_loop(&mut terminal, &rt, &mut app, &mut session_mgr, &mut cross_memory, &tool_cache, &llm_tx, &mut llm_rx);

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    result?;

    Ok(())
}

fn tui_main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rt: &tokio::runtime::Runtime,
    app: &mut app::App,
    session_mgr: &mut session::SessionManager,
    cross_memory: &mut memory::CrossSessionMemory,
    tool_cache: &tool_cache::ToolDocCache,
    llm_tx: &mpsc::UnboundedSender<LlmEvent>,
    llm_rx: &mut mpsc::UnboundedReceiver<LlmEvent>,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Process LLM events
        while let Ok(event) = llm_rx.try_recv() {
            match event {
                LlmEvent::NewRound => {
                    app.start_assistant_message();
                }
                LlmEvent::Token(text) => {
                    app.append_assistant_text(&text);
                }
                LlmEvent::Status(text) => {
                    app.set_status(&text);
                }
                LlmEvent::ToolExecuted {
                    name,
                    args,
                    result,
                } => {
                    app.add_tool_call(&name, &args, &result);

                    // Record tool usage for cross-session memory
                    if name == "i_rs" {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&args) {
                            if let Some(tool) =
                                parsed.get("tool").and_then(|t| t.as_str())
                            {
                                cross_memory.record_tool_use(tool);
                            }
                        }
                    } else {
                        cross_memory.record_tool_use(&name);
                    }
                }
                LlmEvent::Error(text) => {
                    app.add_error(&text);
                }
                LlmEvent::Done(msgs) => {
                    app.finish_processing(Some(msgs.clone()));

                    // Persist conversation to session
                    let session_id = session_mgr
                        .current_id()
                        .unwrap_or_default()
                        .to_string();
                    let records: Vec<serde_json::Value> = app
                        .messages
                        .iter()
                        .map(|m| match m {
                            crate::app::Message::User { text } => {
                                serde_json::json!({"type": "user", "text": text})
                            }
                            crate::app::Message::Assistant { text } => {
                                serde_json::json!({"type": "assistant", "text": text})
                            }
                            crate::app::Message::ToolCall {
                                name,
                                args,
                                result,
                            } => serde_json::json!({
                                "type": "tool_call",
                                "name": name,
                                "args": args,
                                "result": result
                            }),
                            crate::app::Message::Error { text } => {
                                serde_json::json!({"type": "error", "text": text})
                            }
                        })
                        .collect();
                    session_mgr.save_all_messages(&session_id, &records);
                    session_mgr.save_api_messages(&session_id, &msgs);

                    // Rename session based on first user message
                    let needs_rename = session_mgr
                        .current_session()
                        .map(|s| s.title == "新对话" || s.title.is_empty())
                        .unwrap_or(false);
                    if needs_rename {
                        if let Some(first_user) = app.messages.iter().find_map(|m| {
                            if let crate::app::Message::User { text } = m {
                                Some(text.clone())
                            } else {
                                None
                            }
                        }) {
                            session_mgr
                                .rename_session(&session_id, &first_user);
                        }
                    }
                }
            }
        }

        // Handle terminal events
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                        break;
                    }
                    KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                        // Toggle session list
                        app.show_session_list = !app.show_session_list;
                        if app.show_session_list {
                            app.session_list_index = 0;
                            app.session_list = session_mgr.sessions().to_vec();
                        }
                    }
                    KeyCode::Esc if app.show_session_list => {
                        app.show_session_list = false;
                    }
                    KeyCode::Up if app.show_session_list => {
                        app.session_list_index = app
                            .session_list_index
                            .saturating_sub(1);
                    }
                    KeyCode::Down if app.show_session_list => {
                        let max = app.session_list.len().saturating_sub(1);
                        if app.session_list_index < max {
                            app.session_list_index += 1;
                        }
                    }
                    KeyCode::Enter if app.show_session_list => {
                        if let Some(meta) = app
                            .session_list
                            .get(app.session_list_index)
                        {
                            let new_id = meta.id.clone();
                            let is_current = session_mgr
                                .current_id()
                                .map(|id| id == &new_id)
                                .unwrap_or(false);
                            if !is_current {
                                let old_id = session_mgr
                                    .current_id()
                                    .unwrap_or_default()
                                    .to_string();
                                let records: Vec<serde_json::Value> =
                                    app.messages
                                        .iter()
                                        .map(|m| match m {
                                            crate::app::Message::User { text } => {
                                                serde_json::json!({"type": "user", "text": text})
                                            }
                                            crate::app::Message::Assistant { text } => {
                                                serde_json::json!({"type": "assistant", "text": text})
                                            }
                                            crate::app::Message::ToolCall {
                                                name, args, result,
                                            } => serde_json::json!({
                                                "type": "tool_call",
                                                "name": name,
                                                "args": args,
                                                "result": result
                                            }),
                                            crate::app::Message::Error { text } => {
                                                serde_json::json!({"type": "error", "text": text})
                                            }
                                        })
                                        .collect();
                                session_mgr.save_all_messages(&old_id, &records);
                                if let Some(ref msgs) = app.api_messages {
                                    session_mgr.save_api_messages(&old_id, msgs);
                                }

                                // Switch to new session
                                session_mgr.switch_to(&new_id);
                                let loaded = session_mgr.load_app_messages(&new_id, 50);
                                app.messages = loaded;
                                app.api_messages = session_mgr.load_api_messages(&new_id);
                                app.tool_call_count = 0;
                                app.status_text.clear();
                            }
                        }
                        app.show_session_list = false;
                    }
                    KeyCode::Enter if !app.show_session_list => {
                        if !app.input.is_empty() && !app.is_processing() {
                            let text = std::mem::take(&mut app.input);
                            app.add_user_message(&text);

                            // Persist user message to session
                            session_mgr.append_message("user", &text, None);

                            // Build messages for LLM
                            let msgs = crate::llm::build_messages(
                                &app.messages,
                                &text,
                                &app.api_messages,
                                &tool_cache.index_text,
                                &cross_memory.format_hot_tools(tool_cache),
                                &cross_memory.format_user_memory(),
                            );

                            // Spawn LLM chat in background
                            let config = app.config.clone();
                            let tx = llm_tx.clone();
                            rt.spawn(async move {
                                crate::llm::chat_loop(config, msgs, tx).await;
                            });
                        }
                    }
                    KeyCode::Backspace => {
                        if !app.input.is_empty() {
                            app.input.pop();
                        }
                    }
                    KeyCode::Char(c) => {
                        if !app.is_processing() {
                            app.input.push(c);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    Ok(())
}

fn claw_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("无法获取用户主目录")
        .join(".i-rs-claw")
}
