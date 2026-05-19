use crate::app;
use crate::config::Config;
use crate::core;
use crate::llm::LlmEvent;
use chrono;
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::Instant;
use tokio::sync::mpsc;
use owo_colors::OwoColorize;

// ── Helpers ──

/// Serialize app messages to JSON and persist them for a session.
/// Used in multiple places (chat loop, new session, agent switch, session switch).
fn save_session_messages(
    session_mgr: &crate::session::SessionManager,
    session_id: &str,
    messages: &[crate::app::Message],
    api_messages: Option<&[serde_json::Value]>,
) {
    let records: Vec<serde_json::Value> = messages
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
                step,
                total_steps,
            } => serde_json::json!({
                "type": "tool_call",
                "name": name,
                "args": args,
                "result": result,
                "step": step,
                "total_steps": total_steps,
            }),
            crate::app::Message::Error { text } => {
                serde_json::json!({"type": "error", "text": text})
            }
        })
        .collect();
    session_mgr.save_all_messages(session_id, &records);
    if let Some(msgs) = api_messages {
        session_mgr.save_api_messages(session_id, msgs);
    }
}

// =============================================
// TUI subcommand
// =============================================

pub fn run(session_id: Option<&str>) -> anyhow::Result<()> {
    let mut config = Config::load()?;

    // Discover plugins and merge into MCP config BEFORE creating AppCore
    // so that MCP registries are initialized with plugin configs
    if config.plugins_auto_discover {
        let plugin_mgr = crate::plugin::PluginManager::new();
        let plugin_configs = plugin_mgr.to_mcp_configs();
        if !plugin_configs.is_empty() {
            config.mcp_servers.extend(plugin_configs);
        }
    }

    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let mut terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout))?;

    let rt = tokio::runtime::Runtime::new()?;
    let (llm_tx, mut llm_rx) = mpsc::unbounded_channel::<LlmEvent>();

    let mut app = app::App::new(config);

    // Initialize AppCore (session manager, memory, tool cache, skill store)
    let mut app_core = crate::core::AppCore::new(app.config.clone())?;

    // If a specific session ID was requested, try to switch to it
    if let Some(sid) = session_id {
        if !app_core.session_mgr.switch_to(sid) {
            eprintln!("⚠ 未找到会话: {}", sid);
        }
    }

    // Ensure at least one session exists
    if app_core.session_mgr.current_id().is_none() {
        app_core.session_mgr.create_session();
    }

    // Analyze cross-session tool usage from all sessions
    let agent_id = app.current_agent.clone();
    app_core.agent_store.memory_for_mut(&agent_id).analyze_sessions(app_core.session_mgr.sessions(), &app_core.session_mgr);

    // Load messages from current session
    let session_id = app_core.session_mgr.current_id()
        .ok_or_else(|| anyhow::anyhow!("无当前会话，无法加载消息"))?
        .to_string();
    let loaded = app_core.session_mgr.load_app_messages(&session_id, 50);
    app.messages = loaded;
    app.sync_message_timestamps();

    // Check for due reminders at startup
    app.reminder_text = check_reminders();
    if app.reminder_text.is_some() {
        notify_macos("i-rs-claw 提醒", "你有即将到期或已过期的提醒事项");
    }

    if app.messages.is_empty() {
        let onboarding = !app_core.agent_store.memory_for(&app.current_agent).has_user_profile();
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
            app.message_timestamps.push(chrono::Local::now().naive_local());
        } else {
            app.messages.push(app::Message::Assistant {
                text: "你好！我是 i-rs-claw，你的个人数据智能助理。\
                       \n我可以帮你管理健康、财务、任务、媒体等个人信息。\
                       \n试试说：\"记录体重75kg\" 或 \"最近跑步情况如何？\""
                    .to_string(),
            });
            app.message_timestamps.push(chrono::Local::now().naive_local());
        }
    }

    let result = main_loop(
        &mut terminal,
        &rt,
        &mut app,
        &mut app_core,
        &llm_tx,
        &mut llm_rx,
    );

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    // Print styled re-entry command and session summary
    let msg_count = app.messages.len();
    let tool_count = app.tool_call_count;
    let token_display = app.token_usage
        .as_ref()
        .map(|u| format!(" · {} tokens", u.prompt_tokens + u.completion_tokens))
        .unwrap_or_default();

    println!("{}", "✨ 已退出 i-rs-claw".cyan().bold());
    println!("{}", format!("  📊 {} 条消息 · {} 次工具调用{}", msg_count, tool_count, token_display).dimmed());
    if let Some(sid) = app_core.session_mgr.current_id() {
        println!("{} {}", "↻ 重新进入:".yellow(), format!("i-rs-claw tui --session {}", sid).cyan().bold());
    }

    if let Err(e) = &result {
        eprintln!("{} {}", "✗ 错误:".red().bold(), e.to_string().red());
    }

    result
}

fn main_loop(
    terminal: &mut ratatui::Terminal<CrosstermBackend<io::Stdout>>,
    rt: &tokio::runtime::Runtime,
    app: &mut app::App,
    app_core: &mut core::AppCore,
    llm_tx: &mpsc::UnboundedSender<LlmEvent>,
    llm_rx: &mut mpsc::UnboundedReceiver<LlmEvent>,
) -> anyhow::Result<()> {
    let mut last_reminder_check = Instant::now();
    const REMINDER_INTERVAL_SECS: u64 = 120;

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
                    // Only detect/track plan steps in Plan-then-Execute mode
                    if app.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute {
                        let plan_text = app
                            .messages
                            .last()
                            .map(|m| {
                                if let crate::app::Message::Assistant { text: t } = m {
                                    t.clone()
                                } else {
                                    String::new()
                                }
                            })
                            .unwrap_or_default();
                        if !plan_text.is_empty() {
                            app.detect_plan(&plan_text);
                            // Persist plan steps to disk
                            if let Some(sid) = app_core.session_mgr.current_id() {
                                app_core.session_mgr.save_plan_steps(sid, &app.plan_steps);
                            }
                        }
                    }
                }
                LlmEvent::Reasoning(text) => {
                    app.current_reasoning.push_str(&text);
                }
                LlmEvent::Status(text) => {
                    app.set_status(&text);
                    // Track session state transitions
                    if text.starts_with("⚡") || text.contains("并行执行") {
                        if let Some(sid) = app_core.session_mgr.current_id().map(|s| s.to_string()) {
                            app_core.session_mgr.mark_waiting_for_tool(&sid);
                        }
                    }
                }
                LlmEvent::ToolExecuted {
                    name,
                    args,
                    result,
                    step,
                    total_steps,
                } => {
                    app.add_tool_call(&name, &args, &result, step, total_steps);
                    // Mark the next plan step as completed (only in Plan-then-Execute mode)
                    if app.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute {
                        app.mark_next_plan_step_done();
                        // Persist plan progress to disk
                        if let Some(sid) = app_core.session_mgr.current_id() {
                            app_core.session_mgr.save_plan_steps(sid, &app.plan_steps);
                        }
                    }

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
                                app_core.agent_store.memory_for_mut(&app.current_agent).set_user_name(user_name);
                            }
                            if let Some(info) =
                                parsed.get("user_info").and_then(|v| v.as_array())
                            {
                                for item in info {
                                    if let Some(s) =
                                        item.as_str().filter(|s| !s.is_empty())
                                    {
                                        app_core.agent_store.memory_for_mut(&app.current_agent).add_user_info(s);
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
                                        app_core.agent_store.memory_for_mut(&app.current_agent).add_preference(s);
                                    }
                                }
                            }
                        }
                    }

                    // Record tool usage for cross-session memory
                    let agent_id = app.current_agent.clone();
                    if name == "i_rs" {
                        if let Ok(parsed) =
                            serde_json::from_str::<serde_json::Value>(&args)
                        {
                            if let Some(tool) =
                                parsed.get("tool").and_then(|t| t.as_str())
                            {
                                app_core.agent_store.memory_for_mut(&agent_id).record_tool_use(tool);
                            }
                        }
                    } else {
                        app_core.agent_store.memory_for_mut(&agent_id).record_tool_use(&name);
                    }
                }
                LlmEvent::Error(text) => {
                    app.add_error(&text);
                    if let Some(sid) = app_core.session_mgr.current_id().map(|s| s.to_string()) {
                        app_core.session_mgr.mark_error(&sid, &text);
                    }
                }
                LlmEvent::HttpLog {
                    status,
                    duration_ms,
                    model,
                    prompt_tokens,
                    completion_tokens,
                    error,
                    request_body,
                } => {
                    app.add_http_log(app::HttpLog {
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        status,
                        duration_ms,
                        model,
                        prompt_tokens,
                        completion_tokens,
                        error,
                        request_body,
                    });
                }
                LlmEvent::Done(mut msgs, usage) => {
                    // Compress API messages to protect teach docs + fit context
                    app_core.compress_api_messages(&mut msgs, &app.current_agent);

                    app.finish_processing(Some(msgs.clone()));
                    app.token_usage = usage;

                    // Mark session as active (turn completed)
                    if let Some(sid) = app_core.session_mgr.current_id().map(|s| s.to_string()) {
                        app_core.session_mgr.mark_active(&sid);
                    }

                    // Clear persisted plan on completion (if Plan-then-Execute mode)
                    if app.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute {
                        if let Some(sid) = app_core.session_mgr.current_id() {
                            app_core.session_mgr.save_plan_steps(sid, &[]);
                        }
                    }

                    // Persist conversation to session
                    let session_id = match app_core.session_mgr.current_id() {
                        Some(id) => id.to_string(),
                        None => {
                            eprintln!("⚠ 未找到当前会话，跳过持久化");
                            app.finish_processing(None);
                            break;
                        }
                    };
                    save_session_messages(
                        &app_core.session_mgr,
                        &session_id,
                        &app.messages,
                        Some(&msgs),
                    );

                    // Rename session based on first user message
                    let needs_rename = app_core.session_mgr
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
                            app_core.session_mgr.rename_session(&session_id, &first_user);
                        }
                    }

                    // Flush pending memory writes (tool frequency, user info, etc.)
                    app_core.agent_store.memory_for_mut(&app.current_agent).flush();
                }
            }
        }

        // Periodic background reminder check (every 2 minutes, non-blocking)
        {
            let elapsed = last_reminder_check.elapsed().as_secs();
            if elapsed >= REMINDER_INTERVAL_SECS && !app.is_processing() {
                let h = rt.spawn_blocking(check_reminders);
                if let Ok(Some(reminders)) = rt.block_on(h) {
                    app.reminder_text = Some(reminders);
                }
                last_reminder_check = Instant::now();
            }
        }

        // Handle terminal events
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                        let content = if app.selection_mode {
                            // Copy selected message
                            app.selected_message.and_then(|idx| {
                                app.messages.get(idx).map(|m| match m {
                                    crate::app::Message::User { text } => text.clone(),
                                    crate::app::Message::Assistant { text } => text.clone(),
                                    crate::app::Message::ToolCall { name, args, result, .. } =>
                                        format!("Tool: {}\nArgs: {}\nResult: {}", name, args, result),
                                    crate::app::Message::Error { text } => text.clone(),
                                })
                            })
                        } else {
                            // Copy last assistant message
                            app.messages
                                .iter()
                                .rev()
                                .find_map(|m| match m {
                                    crate::app::Message::Assistant { text } if !text.is_empty() => Some(text.clone()),
                                    _ => None,
                                })
                        };
                        if let Some(content) = content {
                            if copy_to_clipboard(&content) {
                                app.copy_feedback = Some("✓ 已复制".to_string());
                            } else {
                                app.copy_feedback = Some("✗ 复制失败".to_string());
                            }
                        } else {
                            app.copy_feedback = Some("无内容可复制".to_string());
                        }
                    }
                    // Ctrl+S: Toggle message selection mode
                    KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                        if !app.messages.is_empty() {
                            app.selection_mode = !app.selection_mode;
                            app.selected_message = if app.selection_mode {
                                Some(app.messages.len().saturating_sub(1))
                            } else {
                                None
                            };
                        }
                    }
                    // Ctrl+H: show keyboard shortcut help panel
                    KeyCode::Char('h') if key.modifiers == KeyModifiers::CONTROL => {
                        app.show_help = !app.show_help;
                    }
                    // Esc/Enter: close help panel
                    KeyCode::Esc | KeyCode::Enter if app.show_help => {
                        app.show_help = false;
                    }
                    // Esc: exit selection mode
                    KeyCode::Esc if app.selection_mode => {
                        app.selection_mode = false;
                        app.selected_message = None;
                    }
                    // q: exit selection mode
                    KeyCode::Char('q') if app.selection_mode => {
                        app.selection_mode = false;
                        app.selected_message = None;
                    }
                    KeyCode::Char('d')
                        if key.modifiers == KeyModifiers::CONTROL && app.selection_mode =>
                    {
                        if let Some(idx) = app.selected_message {
                            let idx = idx.min(app.messages.len().saturating_sub(1));
                            app.messages.remove(idx);
                            app.message_timestamps.remove(idx);
                            app.tool_call_expanded.remove(&idx);
                            // Fix up expanded indices
                            let tc = std::mem::take(&mut app.tool_call_expanded);
                            app.tool_call_expanded = tc.into_iter().map(|i| if i > idx { i - 1 } else { i }).collect();
                            if idx >= app.messages.len() {
                                app.selected_message = if app.messages.is_empty() { None } else { Some(app.messages.len() - 1) };
                            }
                        }
                    }
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
                            app.session_list = app_core.session_mgr.sessions().to_vec();
                        }
                    }
                    KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                        // Save current session, create new one
                        if let Some(old_id) =
                            app_core.session_mgr.current_id().map(|id| id.to_string())
                        {
                            save_session_messages(
                                &app_core.session_mgr,
                                &old_id,
                                &app.messages,
                                app.api_messages.as_deref(),
                            );
                        }
                        app_core.session_mgr.create_session();
                        app.reset_for_new_session();
                        app.show_session_list = false;
                    }
                    KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                        // Toggle HTTP debug sidebar
                        app.show_sidebar = !app.show_sidebar;
                        if app.show_sidebar {
                            app.show_session_list = false;
                        }
                    }
                    KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                        // Toggle agent picker
                        app.show_agent_picker = !app.show_agent_picker;
                        if app.show_agent_picker {
                            app.show_session_list = false;
                            app.show_sidebar = false;
                            app.sidebar_body_idx = None;
                            app.agent_list = app_core.config.agent_ids();
                            app.agent_picker_index = app.agent_list
                                .iter()
                                .position(|id| *id == app.current_agent)
                                .unwrap_or(0);
                        }
                    }
                    // Agent picker: Esc to close
                    KeyCode::Esc if app.show_agent_picker => {
                        app.show_agent_picker = false;
                    }
                    // Agent picker: Enter to switch
                    KeyCode::Enter if app.show_agent_picker => {
                        let agent_id = app.agent_list.get(app.agent_picker_index).cloned();
                        if let Some(ref agent_id) = agent_id {
                            if *agent_id != app.current_agent {
                                // Save current session messages
                                if let Some(old_id) = app_core.session_mgr.current_id().map(|id| id.to_string()) {
                                    save_session_messages(
                                        &app_core.session_mgr,
                                        &old_id,
                                        &app.messages,
                                        app.api_messages.as_deref(),
                                    );
                                }

                                // Switch to new agent
                                app.current_agent = agent_id.clone();
                                app.reset_for_new_session();
                                app.status_text = format!("已切换到 agent: {}", agent_id);

                                // Create new session for this agent
                                app_core.session_mgr.create_session_for(agent_id);

                                // Re-analyze tool usage for the new agent
                                app_core.agent_store.memory_for_mut(agent_id)
                                    .analyze_sessions(app_core.session_mgr.sessions(), &app_core.session_mgr);
                            }
                        }
                        app.show_agent_picker = false;
                    }
                    // Agent picker: Up/Down
                    KeyCode::Up if app.show_agent_picker => {
                        app.agent_picker_index = app.agent_picker_index.saturating_sub(1);
                    }
                    KeyCode::Down if app.show_agent_picker => {
                        let max = app.agent_list.len().saturating_sub(1);
                        if app.agent_picker_index < max {
                            app.agent_picker_index += 1;
                        }
                    }
                    KeyCode::Esc if app.show_session_list => {
                        if app.session_search_mode {
                            // Exit search mode
                            app.session_search_mode = false;
                            app.session_search.clear();
                        } else if app.session_confirm_delete {
                            app.session_confirm_delete = false;
                        } else if !app.session_rename_buf.is_empty() {
                            app.session_rename_buf.clear();
                        } else {
                            app.show_session_list = false;
                        }
                    }
                    // Session list: Ctrl+D delete
                    KeyCode::Char('d')
                        if key.modifiers == KeyModifiers::CONTROL && app.show_session_list =>
                    {
                        app.session_confirm_delete = true;
                    }
                    // Session list: Ctrl+R rename
                    KeyCode::Char('r')
                        if key.modifiers == KeyModifiers::CONTROL && app.show_session_list =>
                    {
                        let q = app.session_search.to_lowercase();
                        let filtered: Vec<&crate::session::SessionMeta> = if q.is_empty() {
                            app.session_list.iter().collect()
                        } else {
                            app.session_list
                                .iter()
                                .filter(|s| s.title.to_lowercase().contains(&q))
                                .collect()
                        };
                        if let Some(meta) = filtered.get(app.session_list_index) {
                            app.session_rename_buf = meta.title.clone();
                        }
                    }
                    // Confirm delete: 'y' to confirm
                    KeyCode::Char('y') if app.session_confirm_delete => {
                        let q = app.session_search.to_lowercase();
                        let filtered: Vec<&crate::session::SessionMeta> = if q.is_empty() {
                            app.session_list.iter().collect()
                        } else {
                            app.session_list
                                .iter()
                                .filter(|s| s.title.to_lowercase().contains(&q))
                                .collect()
                        };
                        if let Some(meta) = filtered.get(app.session_list_index) {
                            let id = meta.id.clone();
                            let is_current = app_core.session_mgr
                                .current_id()
                                .map(|cid| cid == &id)
                                .unwrap_or(false);
                            app_core.session_mgr.delete_session(&id);
                            app.session_confirm_delete = false;
                            app.session_list = app_core.session_mgr.sessions().to_vec();
                            if is_current {
                                // If current session was deleted, reset
                                app.reset_for_new_session();
                            }
                        } else {
                            app.session_confirm_delete = false;
                        }
                    }
                    // Confirm delete: 'n' or any other key to cancel
                    KeyCode::Char('n') if app.session_confirm_delete => {
                        app.session_confirm_delete = false;
                    }
                    KeyCode::Esc if app.show_sidebar && app.sidebar_body_idx.is_some() => {
                        // Close body overlay, keep sidebar open
                        app.sidebar_body_idx = None;
                    }
                    KeyCode::Esc if app.show_sidebar => {
                        app.show_sidebar = false;
                    }
                    // Clear tab completions on Esc
                    KeyCode::Esc if !app.tab_completions.is_empty() => {
                        app.tab_completions.clear();
                        app.tab_completion_index = 0;
                    }
                    KeyCode::Up
                        if !app.show_session_list && app.show_sidebar && app.sidebar_body_idx.is_none() =>
                    {
                        app.sidebar_selected = app.sidebar_selected.saturating_sub(1);
                    }
                    KeyCode::Down
                        if !app.show_session_list && app.show_sidebar && app.sidebar_body_idx.is_none() =>
                    {
                        let max = app.http_logs.len().saturating_sub(1);
                        if app.sidebar_selected < max {
                            app.sidebar_selected += 1;
                        }
                    }
                    KeyCode::Enter
                        if !app.show_session_list && app.show_sidebar && app.sidebar_body_idx.is_none() =>
                    {
                        if !app.http_logs.is_empty() {
                            app.sidebar_body_idx = Some(app.sidebar_selected);
                        }
                    }
                    // Body overlay Up/Down (scroll within the JSON)
                    KeyCode::Up
                        if !app.show_session_list && app.show_sidebar && app.sidebar_body_idx.is_some() =>
                    {
                        app.sidebar_body_scroll =
                            app.sidebar_body_scroll.saturating_sub(1);
                    }
                    KeyCode::Down
                        if !app.show_session_list && app.show_sidebar && app.sidebar_body_idx.is_some() =>
                    {
                        app.sidebar_body_scroll += 1;
                    }
                    KeyCode::Up if app.show_session_list => {
                        app.session_list_index =
                            app.session_list_index.saturating_sub(1);
                    }
                    KeyCode::Down if app.show_session_list => {
                        // In search mode, don't change selection index
                        if !app.session_search_mode {
                            let max = app.session_list.len().saturating_sub(1);
                            if app.session_list_index < max {
                                app.session_list_index += 1;
                            }
                        }
                    }
                    KeyCode::Char('/') if app.show_session_list && !app.session_search_mode => {
                        // Enter search mode
                        app.session_search_mode = true;
                        app.session_search.clear();
                    }
                    KeyCode::Char(c) if app.show_session_list && app.session_search_mode => {
                        // Type to search
                        app.session_search.push(c);
                        app.session_list_index = 0;
                    }
                    KeyCode::Backspace if app.show_session_list && app.session_search_mode => {
                        app.session_search.pop();
                        app.session_list_index = 0;
                    }
                    // Selection mode: navigate messages (Up=older, Down=newer)
                    KeyCode::Up if app.selection_mode => {
                        if let Some(idx) = app.selected_message {
                            if idx > 0 {
                                app.selected_message = Some(idx - 1);
                            }
                        }
                    }
                    KeyCode::Down if app.selection_mode => {
                        if let Some(idx) = app.selected_message {
                            if idx + 1 < app.messages.len() {
                                app.selected_message = Some(idx + 1);
                                // Auto-scroll if newly selected message is not visible
                                let n = app.messages.len();
                                let max_visible = n.saturating_sub(1).saturating_sub(app.scroll_offset);
                                if idx + 1 > max_visible {
                                    app.scroll_offset = n.saturating_sub(1).saturating_sub(idx + 1);
                                }
                            }
                        }
                    }
                    // Selection mode: Space toggles tool call expansion
                    KeyCode::Char(' ') if app.selection_mode => {
                        if let Some(idx) = app.selected_message {
                            if matches!(app.messages.get(idx), Some(crate::app::Message::ToolCall { .. })) {
                                if !app.tool_call_expanded.remove(&idx) {
                                    app.tool_call_expanded.insert(idx);
                                }
                            }
                        }
                    }
                    KeyCode::Up
                        if !app.show_session_list && !app.show_sidebar && !app.is_processing() && !app.selection_mode =>
                    {
                        if app.input.is_empty() {
                            app.scroll_up();
                        } else if let Some(text) = app.navigate_history_up() {
                            app.input = text;
                            app.move_cursor_end();
                        }
                    }
                    KeyCode::Down
                        if !app.show_session_list && !app.show_sidebar && !app.is_processing() && !app.selection_mode =>
                    {
                        if app.input.is_empty() {
                            app.scroll_down();
                        } else if let Some(text) = app.navigate_history_down() {
                            app.input = text;
                            app.move_cursor_end();
                        } else {
                            app.input.clear();
                            app.input_cursor = 0;
                        }
                    }
                    KeyCode::Enter if app.show_session_list => {
                        // If renaming, confirm rename
                        if !app.session_rename_buf.is_empty() {
                            let q = app.session_search.to_lowercase();
                            let filtered: Vec<&crate::session::SessionMeta> = if q.is_empty() {
                                app.session_list.iter().collect()
                            } else {
                                app.session_list
                                    .iter()
                                    .filter(|s| s.title.to_lowercase().contains(&q))
                                    .collect()
                            };
                            if let Some(meta) = filtered.get(app.session_list_index) {
                                let title = std::mem::take(&mut app.session_rename_buf);
                                if !title.trim().is_empty() {
                                    app_core.session_mgr.rename_session(&meta.id, title.trim());
                                }
                                app.session_list = app_core.session_mgr.sessions().to_vec();
                            } else {
                                app.session_rename_buf.clear();
                            }
                            // Keep the session list open after rename
                            break;
                        }

                        // Build filtered list to find the actual session ID
                        let q = app.session_search.to_lowercase();
                        let filtered: Vec<&crate::session::SessionMeta> = if q.is_empty() {
                            app.session_list.iter().collect()
                        } else {
                            app.session_list
                                .iter()
                                .filter(|s| s.title.to_lowercase().contains(&q))
                                .collect()
                        };

                        if app.session_search_mode {
                            app.session_search_mode = false;
                        }

                        if let Some(meta) = filtered.get(app.session_list_index) {
                            let new_id = meta.id.clone();
                            let is_current = app_core.session_mgr
                                .current_id()
                                .map(|id| id == &new_id)
                                .unwrap_or(false);
                            if !is_current {
                                let old_id = app_core.session_mgr
                                    .current_id()
                                    .unwrap_or_default()
                                    .to_string();
                                    save_session_messages(
                                        &app_core.session_mgr,
                                        &old_id,
                                        &app.messages,
                                        app.api_messages.as_deref(),
                                    );

                                // Switch to new session
                                app_core.session_mgr.switch_to(&new_id);
                                let loaded =
                                    app_core.session_mgr.load_app_messages(&new_id, 50);
                                app.messages = loaded;
                                app.sync_message_timestamps();
                                app.api_messages =
                                    app_core.session_mgr.load_api_messages(&new_id);
                                app.tool_call_count = 0;
                                app.status_text.clear();
                                app.token_usage = None;
                                app.plan_steps =
                                    app_core.session_mgr.load_plan_steps(&new_id);
                            }
                        }
                        app.show_session_list = false;
                    }
                    KeyCode::Enter => {
                        if key.modifiers == KeyModifiers::ALT {
                            if !app.is_processing() {
                                app.insert_char('\n');
                            }
                        } else if !app.input.is_empty() && !app.is_processing() {
                            let text = std::mem::take(&mut app.input);
                            app.input_cursor = 0;
                            app.commit_input_to_history(&text);
                            app.add_user_message(&text);

                            // Persist user message to session
                            app_core.session_mgr.append_message("user", &text, None);

                            // Build messages for LLM
                            let msgs = app_core.build_messages_for(
                                &app.messages,
                                &text,
                                &app.api_messages,
                                app.reminder_text.as_deref(),
                                &app.current_agent,
                            );

                            // Spawn LLM chat in background
                            app_core.spawn_chat_for(rt, llm_tx.clone(), msgs, &app.current_agent);
                        }
                    }
                    KeyCode::Backspace => {
                        if !app.input.is_empty() {
                            app.delete_before_cursor();
                            // Clear tab completions on edit
                            if !app.tab_completions.is_empty() {
                                app.tab_completions.clear();
                                app.tab_completion_index = 0;
                            }
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
                    KeyCode::Tab => {
                        if !app.is_processing() && !app.input.is_empty() {
                            let completions = crate::completion::get_completions(&app.input, app.input_cursor);
                            if !completions.is_empty() {
                                if app.tab_completions.is_empty() {
                                    app.tab_completions = completions;
                                    app.tab_completion_index = 0;
                                } else {
                                    // Cycle forward
                                    app.tab_completion_index = (app.tab_completion_index + 1) % app.tab_completions.len();
                                }
                                // Replace input with selected completion + space
                                let selected = &app.tab_completions[app.tab_completion_index];
                                let before = &app.input[..app.input_cursor];
                                let after = &app.input[app.input_cursor..];
                                // Find last word boundary for replacement
                                let word_start = before.rfind(|c: char| c.is_whitespace()).map(|i| i + 1).unwrap_or(0);
                                app.input = format!("{}{} {}", &before[..word_start], selected, after);
                                app.input_cursor = word_start + selected.len() + 1;
                            }
                        }
                    }
                    KeyCode::BackTab => {
                        // Shift+Tab: cycle backward
                        if !app.is_processing() && !app.tab_completions.is_empty() {
                            let len = app.tab_completions.len();
                            app.tab_completion_index = if app.tab_completion_index == 0 {
                                len.saturating_sub(1)
                            } else {
                                app.tab_completion_index - 1
                            };
                            let selected = &app.tab_completions[app.tab_completion_index];
                            let before = &app.input[..app.input_cursor];
                            let after = &app.input[app.input_cursor..];
                            let word_start = before.rfind(|c: char| c.is_whitespace()).map(|i| i + 1).unwrap_or(0);
                            app.input = format!("{}{} {}", &before[..word_start], selected, after);
                            app.input_cursor = word_start + selected.len() + 1;
                        }
                    }
                    KeyCode::Char(c) => {
                        if !app.is_processing() {
                            // Clear tab completions when user types
                            if !app.tab_completions.is_empty() {
                                app.tab_completions.clear();
                                app.tab_completion_index = 0;
                            }
                            app.insert_char(c);
                        }
                    }
                    _ => {}
                },
                Event::Mouse(mouse) => {
                    if app.sidebar_body_idx.is_some() {
                        match mouse.kind {
                            MouseEventKind::ScrollDown => {
                                app.sidebar_body_scroll += 1;
                            }
                            MouseEventKind::ScrollUp => {
                                app.sidebar_body_scroll = app
                                    .sidebar_body_scroll
                                    .saturating_sub(1);
                            }
                            _ => {}
                        }
                    } else if !app.is_processing() && !app.show_sidebar {
                        match mouse.kind {
                            MouseEventKind::ScrollDown => app.scroll_up(),
                            MouseEventKind::ScrollUp => app.scroll_down(),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Check for due/overdue reminders via `i-rs remind list --json`.
/// Returns a formatted string listing all due reminders, or None if none found.
fn check_reminders() -> Option<String> {
    let output = std::process::Command::new("i-rs")
        .arg("remind")
        .arg("list")
        .arg("--json")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.as_ref()).ok()?;

    let items = parsed.get("data")?.as_array()?;

    let due: Vec<String> = items
        .iter()
        .filter(|item| {
            let is_done = item
                .get("is_done")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            if is_done {
                return false;
            }
            let days = item
                .get("days_until_event")
                .and_then(|v| v.as_i64())
                .unwrap_or(1);
            days <= 0
        })
        .map(|item| {
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("未知");
            let title = item
                .get("title")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let _date = item
                .get("event_date")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let days = item
                .get("days_until_event")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            match title {
                Some(t) => {
                    if days == 0 {
                        format!("  - {}「{}」（今天到期）", t, name)
                    } else {
                        format!("  - {}「{}」（已过期 {} 天）", t, name, days.abs())
                    }
                }
                None => {
                    if days == 0 {
                        format!("  - {}（今天到期）", name)
                    } else {
                        format!("  - {}（已过期 {} 天）", name, days.abs())
                    }
                }
            }
        })
        .collect();

    if due.is_empty() {
        return None;
    }

    notify_macos("i-rs-claw 提醒", &format!("你有 {} 个待处理提醒", due.len()));
    Some(due.join("\n"))
}

/// Send a macOS notification via osascript.
fn notify_macos(title: &str, message: &str) {
    let _ = std::process::Command::new("osascript")
        .args(["-e", &format!(
            r###"display notification "{}" with title "{}""###,
            message, title
        )])
        .output();
}

/// Copy text to system clipboard using platform-specific command.
fn copy_to_clipboard(text: &str) -> bool {
    let cmd = if cfg!(target_os = "macos") {
        ("pbcopy", &[] as &[&str])
    } else if cfg!(target_os = "linux") {
        // Prefer wl-copy (Wayland), fallback to xclip (X11)
        if std::process::Command::new("wl-copy").output().is_ok() {
            ("wl-copy", &[] as &[&str])
        } else {
            ("xclip", &["-selection", "clipboard"] as &[&str])
        }
    } else if cfg!(target_os = "windows") {
        ("clip", &[] as &[&str])
    } else {
        return false;
    };

    std::process::Command::new(cmd.0)
        .args(cmd.1)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .take()
                .and_then(|mut stdin| {
                    stdin.write_all(text.as_bytes()).ok()
                });
            child.wait_with_output()
        })
        .map(|output| output.status.success())
        .unwrap_or(false)
}
