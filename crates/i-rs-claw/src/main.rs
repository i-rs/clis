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
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Terminal;
use std::io::{self, Write};
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
    Tui {
        /// Resume a specific session by ID
        #[arg(long)]
        session: Option<String>,
    },
    /// Interactive configuration wizard
    Config,
    /// Interactive tool enable/disable
    Tools,
    /// List and manage sessions
    Session {
        /// List all sessions
        #[arg(long)]
        list: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Tui { session: None }) {
        Command::Tui { session } => run_tui(session.as_deref()),
        Command::Config => run_config(),
        Command::Tools => run_tools(),
        Command::Session { list: true } => run_session_list(),
        Command::Session { list: false } => run_session_list(),
    }
}

// =============================================
// Interactive Config Wizard
// =============================================

fn run_config() -> anyhow::Result<()> {
    let config_path = claw_dir().join("config.toml");
    let mut cfg = if config_path.exists() {
        Config::load().unwrap_or_else(|_| Config::new())
    } else {
        println!("未发现配置文件，开始交互式设置...\n");
        Config::new()
    };

    // ── API Key ──
    let current = if cfg.api_key.is_empty() {
        String::new()
    } else {
        format!(" [{}...{}]", &cfg.api_key[..4.min(cfg.api_key.len())], &cfg.api_key[cfg.api_key.len().saturating_sub(4)..])
    };
    print!("API Key{}: ", current);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.api_key = trimmed;
    }

    // ── Base URL ──
    print!("Base URL [{}]: ", cfg.base_url);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.base_url = trimmed;
    }

    // ── Model ──
    print!("Model [{}]: ", cfg.model);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.model = trimmed;
    }

    // ── Save ──
    if cfg.api_key.is_empty() {
        anyhow::bail!("API Key 不能为空，配置未保存");
    }

    cfg.save()?;
    let total_tools = crate::tools::search::TOOL_INDEX.len();
    let enabled_count = if cfg.enabled_tools.is_empty() {
        total_tools
    } else {
        cfg.enabled_tools.len()
    };

    println!("\n配置摘要：");
    println!("  API Key: {}...{}", &cfg.api_key[..4.min(cfg.api_key.len())], &cfg.api_key[cfg.api_key.len().saturating_sub(4)..]);
    println!("  Base URL: {}", cfg.base_url);
    println!("  Model: {}", cfg.model);
    println!("  工具: {} ({} 个 / 总 {} 个)",
        if cfg.enabled_tools.is_empty() { "全部启用" } else { "部分启用" },
        enabled_count,
        total_tools,
    );
    println!("  运行 `i-rs-claw tools` 管理工具开关");

    Ok(())
}

// =============================================
// Tools subcommand
// =============================================

fn run_tools() -> anyhow::Result<()> {
    let mut cfg = Config::load()?;
    let all_tools: Vec<&str> = crate::tools::search::TOOL_INDEX.iter().map(|(n, _)| *n).collect();
    let total = all_tools.len();

    // ── TUI setup ──
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut selection: usize = 0;
    let mut dirty = false;

    let result: anyhow::Result<()> = (|| {
        let mut list_state = ratatui::widgets::ListState::default();
        loop {
            list_state.select(Some(selection));
            terminal.draw(|f| {
                let area = f.area();

                // Layout: title + list + footer
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Min(1),
                        Constraint::Length(1),
                    ])
                    .split(area);

                // Title bar
                let checked_count = if cfg.enabled_tools.is_empty() {
                    total
                } else {
                    cfg.enabled_tools.len()
                };
                let title = Line::from(Span::styled(
                    format!(" ✦ 工具管理  [{}✓ / {}总]  ↑↓选择  Space切换  Enter保存  Esc取消", checked_count, total),
                    Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD),
                ));
                f.render_widget(title, chunks[0]);

                // Tool list
                let items: Vec<ListItem> = all_tools.iter().map(|name| {
                    let checked = cfg.enabled_tools.is_empty() || cfg.enabled_tools.contains(*name);
                    let checkbox = if checked { "[✓]" } else { "[ ]" };
                    let desc = crate::tools::search::TOOL_INDEX
                        .iter()
                        .find(|(n, _)| n == name)
                        .map(|(_, d)| *d)
                        .unwrap_or("");
                    let text = format!(" {} {}", checkbox, name);
                    let line = Line::from(vec![
                        Span::styled(text, Style::default().fg(if checked { Color::Green } else { Color::DarkGray }).add_modifier(if checked { Modifier::BOLD } else { Modifier::empty() })),
                        Span::styled(format!("  — {}", desc), Style::default().fg(Color::DarkGray)),
                    ]);
                    ListItem::new(line)
                }).collect();

                let list = List::new(items)
                    .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::REVERSED))
                    .block(Block::default().borders(Borders::NONE));

                f.render_stateful_widget(list, chunks[1], &mut ratatui::widgets::ListState::default().with_selected(Some(selection)));
            })?;

            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Up => selection = selection.saturating_sub(1),
                    KeyCode::Down if selection + 1 < total => selection += 1,
                    KeyCode::Char(' ') => {
                        dirty = true;
                        let name = all_tools[selection];
                        if cfg.enabled_tools.is_empty() {
                            cfg.enabled_tools = all_tools.iter().map(|s| s.to_string()).collect();
                        }
                        if cfg.enabled_tools.contains(name) {
                            cfg.enabled_tools.remove(name);
                        } else {
                            cfg.enabled_tools.insert(name.to_string());
                        }
                    }
                    KeyCode::Enter => break Ok(()),
                    KeyCode::Esc | KeyCode::Char('q') => {
                        dirty = false;
                        break Ok(());
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    })();

    // ── TUI teardown ──
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    result?;

    // Save if modified
    if dirty {
        if cfg.enabled_tools.len() == total {
            cfg.enabled_tools.clear();
        }
        if cfg.enabled_tools.is_empty() {
            println!("✓ 已启用全部 {} 个工具", total);
        } else {
            println!("✓ 已启用 {} 个工具 (停用 {} 个)", cfg.enabled_tools.len(), total - cfg.enabled_tools.len());
        }
        cfg.save()?;
    } else {
        println!("✓ 未做修改");
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
// TUI subcommand
// =============================================

fn run_tui(session_id: Option<&str>) -> anyhow::Result<()> {
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

    // If a specific session ID was requested, try to switch to it
    if let Some(sid) = session_id {
        if !session_mgr.switch_to(sid) {
            eprintln!("⚠ 未找到会话: {}", sid);
        }
    }

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

    if app.messages.is_empty() {
        let onboarding = !cross_memory.has_user_profile();
        if onboarding {
            app.messages.push(app::Message::Assistant {
                text: concat!(
                    "你好！我是 i-rs-claw，你的个人数据智能助理 🎉\n\n",
                    "初次见面，我想更好地了解你！\n",
                    "请问我怎么称呼你呢？你平时有什么兴趣爱好？\n",
                    "比如你喜欢跑步、健身、读书、看电影，还是有什么特别的日常生活习惯？\n\n",
                    "告诉我这些，我可以更贴心地帮你管理数据 😊",
                ).to_string(),
            });
        } else {
            app.messages.push(app::Message::Assistant {
                text: "你好！我是 i-rs-claw，你的个人数据智能助理。\
                       \n我可以帮你管理健康、财务、任务、媒体等个人信息。\
                       \n试试说：\"记录体重75kg\" 或 \"最近跑步情况如何？\""
                    .to_string(),
            });
        }
    }

    let result = tui_main_loop(&mut terminal, &rt, &mut app, &mut session_mgr, &mut cross_memory, &tool_cache, &llm_tx, &mut llm_rx);

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    // Print re-entry command so user can resume later
    if let Some(sid) = session_mgr.current_id() {
        println!("重新进入会话: i-rs-claw tui --session {}", sid);
    }

    if let Err(e) = &result {
        eprintln!("错误: {}", e);
    }

    result
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
                LlmEvent::Done(msgs, usage) => {
                    app.finish_processing(Some(msgs.clone()));
                    app.token_usage = usage;

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
                    KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                        // Save current session, create new one
                        if let Some(old_id) = session_mgr.current_id().map(|id| id.to_string()) {
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
                        }
                        session_mgr.create_session();
                        app.reset_for_new_session();
                        app.show_session_list = false;
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
                    KeyCode::Up if !app.show_session_list && !app.is_processing() => {
                        if let Some(text) = app.navigate_history_up() {
                            app.input = text;
                            app.move_cursor_end();
                        }
                    }
                    KeyCode::Down if !app.show_session_list && !app.is_processing() => {
                        if let Some(text) = app.navigate_history_down() {
                            app.input = text;
                            app.move_cursor_end();
                        } else {
                            app.input.clear();
                            app.input_cursor = 0;
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
                                app.token_usage = None;
                            }
                        }
                        app.show_session_list = false;
                    }
                    KeyCode::Enter if !app.show_session_list => {
                        if !app.input.is_empty() && !app.is_processing() {
                            let text = std::mem::take(&mut app.input);
                            app.input_cursor = 0;
                            app.commit_input_to_history(&text);
                            app.add_user_message(&text);

                            // Persist user message to session
                            session_mgr.append_message("user", &text, None);

                            // Build messages for LLM
                            let msgs = crate::llm::build_messages(
                                &app.messages,
                                &text,
                                &app.api_messages,
                                &app.tool_index_text,
                                &cross_memory.format_hot_tools(tool_cache),
                                &cross_memory.format_user_memory(),
                                &cross_memory.format_user_profile(),
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
                            app.delete_before_cursor();
                        }
                    }
                    KeyCode::Left => {
                        app.move_cursor_left();
                    }
                    KeyCode::Right => {
                        app.move_cursor_right();
                    }
                    KeyCode::Home => {
                        app.move_cursor_home();
                    }
                    KeyCode::End => {
                        app.move_cursor_end();
                    }
                    KeyCode::Char(c) => {
                        if !app.is_processing() {
                            app.insert_char(c);
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
