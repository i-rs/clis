use crate::app;
use crate::config::Config;
use crate::llm::LlmEvent;
use owo_colors::OwoColorize;
use ratatui::backend::CrosstermBackend;
use std::io;
use tokio::sync::mpsc;

mod clipboard;
mod handlers;
mod main_loop;
mod reminders;

pub fn run(session_id: Option<&str>) -> anyhow::Result<()> {
    let mut config = Config::load()?;

    // Discover plugins and merge into MCP config BEFORE creating AppCore
    // so that MCP registries are initialized with plugin configs
    if config.plugins_auto_discover {
        let plugin_mgr = crate::plugin::PluginManager::new();
        let mut plugin_configs = plugin_mgr.to_mcp_configs();
        // Filter out plugins that are disabled in config
        if !config.disabled_plugins.is_empty() {
            plugin_configs.retain(|pc| {
                let plugin_name = pc.name.strip_prefix("plugin:").unwrap_or(&pc.name);
                !config.disabled_plugins.contains(&plugin_name.to_string())
            });
        }
        if !plugin_configs.is_empty() {
            config.mcp_servers.extend(plugin_configs);
        }
    }

    // Setup terminal (RAII 守卫会在 ? 错误/panic 时自动恢复)
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture,
        crossterm::event::EnableBracketedPaste
    )?;
    let _terminal_guard = TerminalGuard::new();

    // Install panic hook to restore terminal on crash.
    // 注意：守卫的 Drop 也会执行恢复，这里 hook 仅用于先尝试在 panic 信息打印前
    // 把终端切回正常模式，避免 panic 信息被 raw mode 截断。
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut stdout = io::stdout();
        let _ = crossterm::execute!(
            stdout,
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture,
            crossterm::event::DisableBracketedPaste
        );
        let _ = crossterm::terminal::disable_raw_mode();
        default_hook(info);
    }));
    let mut terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout))?;

    let rt = tokio::runtime::Runtime::new()?;
    let (llm_tx, mut llm_rx) = mpsc::unbounded_channel::<LlmEvent>();

    let mut app = app::App::new(config);

    // Initialize AppCore (session manager, memory, tool cache, skill store, stats)
    let mut app_core = crate::core::AppCore::new(app.config.clone())?;

    // Clean up expired stats records on startup
    if app.config.stats.enabled && app.config.stats.keep_days > 0 {
        app_core.stats_manager.cleanup(app.config.stats.keep_days);
    }

    // If a specific session ID was requested, try to switch to it
    if let Some(sid) = session_id
        && !app_core.session_mgr.switch_to(sid)
    {
        // Show error in-app since stderr is invisible in alternate screen
        app.messages.push(app::Message::Error {
            text: format!("未找到会话: {}", sid),
        });
        app.message_timestamps
            .push(chrono::Local::now().naive_local());
    }

    // Ensure at least one session exists
    if app_core.session_mgr.current_id().is_none() {
        app_core.session_mgr.create_session();
    }

    // Analyze cross-session tool usage from all sessions
    let agent_id = app.current_agent.clone();
    app_core
        .agent_store
        .memory_for_mut(&agent_id)
        .analyze_sessions(app_core.session_mgr.sessions(), &app_core.session_mgr);

    // Load messages from current session
    let session_id = app_core
        .session_mgr
        .current_id()
        .ok_or_else(|| anyhow::anyhow!("无当前会话，无法加载消息"))?
        .to_string();
    let loaded = app_core.session_mgr.load_app_messages(&session_id, 200);
    app.messages = loaded;
    app.sync_message_timestamps();

    // Check for due reminders at startup
    app.reminder_text = reminders::check_reminders();
    if let Some(ref reminder_text) = app.reminder_text {
        let count = reminder_text.lines().count();
        reminders::notify_reminders(count);
    }

    if app.messages.is_empty() {
        let onboarding = !app_core
            .agent_store
            .memory_for(&app.current_agent)
            .has_user_profile();
        if onboarding {
            app.messages.push(app::Message::Assistant {
                text: concat!(
                    "你好，我是 i-rs-claw，你的个人数据助理。\n\n",
                    "初次见面！怎么称呼你？有什么我可以帮你的？",
                )
                .to_string(),
                reasoning: String::new(),
                        token_usage: None,
            });
            app.message_timestamps
                .push(chrono::Local::now().naive_local());
        } else {
            app.messages.push(app::Message::Assistant {
                text: "你好，有什么可以帮你的？".to_string(),
                reasoning: String::new(),
                        token_usage: None,
            });
            app.message_timestamps
                .push(chrono::Local::now().naive_local());
        }
    }

    let result = main_loop::main_loop(
        &mut terminal,
        &rt,
        &mut app,
        &mut app_core,
        &llm_tx,
        &mut llm_rx,
    );

    // 退出前持久化当前会话的消息和 API 缓存，否则下次 --session 加载会丢失数据
    if let Some(sid) = app_core.session_mgr.current_id().map(|s| s.to_string()) {
        crate::tui::clipboard::save_session_messages(
            &app_core.session_mgr,
            &sid,
            &app.messages,
            app.api_messages.as_deref(),
        );
    }

    app_core.shutdown();

    // 显式 disarm：main_loop 之后由我们负责控制顺序，守卫不再做事
    _terminal_guard.disarm();
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture,
        crossterm::event::DisableBracketedPaste
    )?;

    // Print styled re-entry command and session summary
    let msg_count = app.messages.len();
    let tool_count = app.tool_call_count;
    let stats = app_core.stats_manager.today_summary();
    let stats_display = if stats.requests > 0 {
        format!(
            " · 今日: {}次 · {} tok · ${:.4}",
            stats.requests, stats.tokens, stats.cost_usd
        )
    } else {
        String::new()
    };

    println!("{}", "✨ 已退出 i-rs-claw".cyan().bold());
    println!(
        "{}",
        format!(
            "  📊 {} 条消息 · {} 次工具调用{}",
            msg_count, tool_count, stats_display
        )
        .dimmed()
    );
    if let Some(sid) = app_core.session_mgr.current_id() {
        println!(
            "{} {}",
            "↻ 重新进入:".yellow(),
            format!("i-rs-claw tui --session {}", sid).cyan().bold()
        );
    }

    if let Err(e) = &result {
        eprintln!("{} {}", "✗ 错误:".red().bold(), e.to_string().red());
    }

    result
}

/// RAII 守卫：Drop 时还原终端状态（raw mode / alt screen / mouse / paste）。
/// 任何在创建之后发生的 `?` 错误或 panic 都会通过 Drop 自动恢复终端。
struct TerminalGuard {
    active: bool,
}

impl TerminalGuard {
    fn new() -> Self {
        Self { active: true }
    }

    /// 主动释放守卫（表示已手动完成还原）。Drop 时不再做事。
    fn disarm(mut self) {
        self.active = false;
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let mut stdout = io::stdout();
        let _ = crossterm::execute!(
            stdout,
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture,
            crossterm::event::DisableBracketedPaste
        );
        let _ = crossterm::terminal::disable_raw_mode();
    }
}
