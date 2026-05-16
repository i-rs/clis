use crate::app;
use crate::config::Config;
use crate::llm::LlmEvent;
use crate::memory::CrossSessionMemory;
use crate::session::SessionManager;
use crate::skill_store::SkillStore;
use crate::tool_cache::ToolDocCache;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use std::io;
use tokio::sync::mpsc;

// =============================================
// TUI subcommand
// =============================================

pub fn run(session_id: Option<&str>) -> anyhow::Result<()> {
    let config = Config::load()?;

    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout))?;

    let rt = tokio::runtime::Runtime::new()?;
    let (llm_tx, mut llm_rx) = mpsc::unbounded_channel::<LlmEvent>();

    let mut app = app::App::new(config);

    // Initialize tool doc cache, session manager, and cross-session memory
    let claw_dir = claw_dir().join("claw");
    let tool_cache = ToolDocCache::new(claw_dir.clone());
    let skill_store = SkillStore::new(claw_dir.clone());
    let mut session_mgr = SessionManager::new(claw_dir.clone());
    let mut cross_memory = CrossSessionMemory::new(claw_dir);

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
                )
                .to_string(),
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

    let result = main_loop(
        &mut terminal,
        &rt,
        &mut app,
        &mut session_mgr,
        &mut cross_memory,
        &tool_cache,
        &skill_store,
        &llm_tx,
        &mut llm_rx,
    );

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

fn main_loop(
    terminal: &mut ratatui::Terminal<CrosstermBackend<io::Stdout>>,
    rt: &tokio::runtime::Runtime,
    app: &mut app::App,
    session_mgr: &mut SessionManager,
    cross_memory: &mut CrossSessionMemory,
    tool_cache: &ToolDocCache,
    skill_store: &SkillStore,
    llm_tx: &mpsc::UnboundedSender<LlmEvent>,
    llm_rx: &mut mpsc::UnboundedReceiver<LlmEvent>,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| crate::ui::render(f, app))?;

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

                    // Save user information from update_user_memory tool
                    if name == "update_user_memory" {
                        if let Ok(parsed) =
                            serde_json::from_str::<serde_json::Value>(&args)
                        {
                            if let Some(user_name) = parsed
                                .get("user_name")
                                .and_then(|v| v.as_str())
                                .filter(|s| !s.is_empty())
                            {
                                cross_memory.set_user_name(user_name);
                            }
                            if let Some(info) =
                                parsed.get("user_info").and_then(|v| v.as_array())
                            {
                                for item in info {
                                    if let Some(s) =
                                        item.as_str().filter(|s| !s.is_empty())
                                    {
                                        cross_memory.add_user_info(s);
                                    }
                                }
                            }
                            if let Some(prefs) = parsed
                                .get("preferences")
                                .and_then(|v| v.as_array())
                            {
                                for item in prefs {
                                    if let Some(s) =
                                        item.as_str().filter(|s| !s.is_empty())
                                    {
                                        cross_memory.add_preference(s);
                                    }
                                }
                            }
                        }
                    }

                    // Record tool usage for cross-session memory
                    if name == "i_rs" {
                        if let Ok(parsed) =
                            serde_json::from_str::<serde_json::Value>(&args)
                        {
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
                            session_mgr.rename_session(&session_id, &first_user);
                        }
                    }
                }
            }
        }

        // Handle terminal events
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Char('c')
                        if key.modifiers == KeyModifiers::CONTROL =>
                    {
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
                        if let Some(old_id) =
                            session_mgr.current_id().map(|id| id.to_string())
                        {
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
                        app.session_list_index =
                            app.session_list_index.saturating_sub(1);
                    }
                    KeyCode::Down if app.show_session_list => {
                        let max = app.session_list.len().saturating_sub(1);
                        if app.session_list_index < max {
                            app.session_list_index += 1;
                        }
                    }
                    KeyCode::Up
                        if !app.show_session_list && !app.is_processing() =>
                    {
                        if app.input.is_empty() {
                            app.scroll_up(3);
                        } else if let Some(text) = app.navigate_history_up() {
                            app.input = text;
                            app.move_cursor_end();
                        }
                    }
                    KeyCode::Down
                        if !app.show_session_list && !app.is_processing() =>
                    {
                        if app.input.is_empty() {
                            app.scroll_down(3);
                        } else if let Some(text) = app.navigate_history_down() {
                            app.input = text;
                            app.move_cursor_end();
                        } else {
                            app.input.clear();
                            app.input_cursor = 0;
                        }
                    }
                    KeyCode::Enter if app.show_session_list => {
                        if let Some(meta) =
                            app.session_list.get(app.session_list_index)
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
                                session_mgr.save_all_messages(&old_id, &records);
                                if let Some(ref msgs) = app.api_messages {
                                    session_mgr.save_api_messages(&old_id, msgs);
                                }

                                // Switch to new session
                                session_mgr.switch_to(&new_id);
                                let loaded =
                                    session_mgr.load_app_messages(&new_id, 50);
                                app.messages = loaded;
                                app.api_messages =
                                    session_mgr.load_api_messages(&new_id);
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
                                &skill_store.format_skills(),
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
