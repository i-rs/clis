use crate::app;
use crate::config::Config;
use crate::llm::LlmEvent;
use ratatui::backend::CrosstermBackend;
use std::io;
use tokio::sync::mpsc;
use owo_colors::OwoColorize;

mod clipboard;
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

    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
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
        && !app_core.session_mgr.switch_to(sid) {
            eprintln!("⚠ 未找到会话: {}", sid);
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
    app.reminder_text = reminders::check_reminders();
    if app.reminder_text.is_some() {
        reminders::notify_macos("i-rs-claw 提醒", "你有即将到期或已过期的提醒事项");
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

    let result = main_loop::main_loop(
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

    // Flush buffered token statistics before exit
    app_core.stats_manager.flush();

    // Print styled re-entry command and session summary
    let msg_count = app.messages.len();
    let tool_count = app.tool_call_count;
    let stats = app_core.stats_manager.today_summary();
    let stats_display = if stats.requests > 0 {
        format!(" · 今日: {}次 · {} tok · ${:.4}", stats.requests, stats.tokens, stats.cost_usd)
    } else {
        String::new()
    };

    println!("{}", "✨ 已退出 i-rs-claw".cyan().bold());
    println!("{}", format!("  📊 {} 条消息 · {} 次工具调用{}", msg_count, tool_count, stats_display).dimmed());
    if let Some(sid) = app_core.session_mgr.current_id() {
        println!("{} {}", "↻ 重新进入:".yellow(), format!("i-rs-claw tui --session {}", sid).cyan().bold());
    }

    if let Err(e) = &result {
        eprintln!("{} {}", "✗ 错误:".red().bold(), e.to_string().red());
    }

    result
}
