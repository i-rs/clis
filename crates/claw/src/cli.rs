use crate::config::Config;
use crate::session::SessionManager;
use crossterm::event::{self, Event, KeyCode};
#[cfg(feature = "dashboard")]
use owo_colors::OwoColorize;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Terminal;
use std::io::{self, Write};

// =============================================
// Interactive Config Wizard
// =============================================

pub fn run_config() -> anyhow::Result<()> {
    let config_path = claw_dir().join("config.toml");
    let mut cfg = if config_path.exists() {
        Config::load().unwrap_or_else(|_| Config::new())
    } else {
        println!("未发现配置文件，开始交互式设置...\n");
        Config::new()
    };

    // ── Provider Selection ──
    let provider_names: Vec<&str> = crate::provider::ProviderKind::all()
        .iter()
        .map(|p| p.as_str())
        .collect();
    let provider_default = if provider_names.contains(&cfg.provider.as_str()) {
        cfg.provider.clone()
    } else {
        "openai".to_string()
    };
    print!("Provider [{}] ({}): ",
        provider_default,
        provider_names.join("/"));
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.provider = trimmed;
    }

    // ── API Key ──
    let current = if cfg.api_key.is_empty() {
        String::new()
    } else {
        format!(
            " [{}...{}]",
            &cfg.api_key[..4.min(cfg.api_key.len())],
            &cfg.api_key[cfg.api_key.len().saturating_sub(4)..]
        )
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

    // ── Search API Key (optional) ──
    let search_current = cfg.search_api_key.as_ref().map(|k| {
        if k.is_empty() {
            String::new()
        } else {
            format!(" [{}...{}]", &k[..4.min(k.len())], &k[k.len().saturating_sub(4)..])
        }
    }).unwrap_or_default();
    print!("Search API Key{} (留空使用 DuckDuckGo): ", search_current);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.search_api_key = Some(trimmed);
    }

    // ── Search Base URL (optional) ──
    let search_url_default = cfg.search_base_url.as_deref().unwrap_or("DuckDuckGo (free)");
    print!("Search Base URL [{}]: ", search_url_default);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.search_base_url = Some(trimmed);
    }

    // ── MCP Servers (optional) ──
    let mcp_count = cfg.mcp_servers.len();
    println!("\n  MCP 服务器 (当前 {} 个):", mcp_count);
    println!("  MCP (Model Context Protocol) 允许连接外部工具服务器。");
    println!("  配置格式: name|command|arg1 arg2|KEY=VAL");
    println!("  例如: filesystem|npx|-y @modelcontextprotocol/server-filesystem /path");
    println!("  留空则跳过 MCP 配置，可后续在配置文件中修改。");
    print!("添加 MCP 服务器 (留空跳过): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let mcp_input = input.trim().to_string();
    if !mcp_input.is_empty() {
        if let Some(server) = parse_mcp_server(&mcp_input) {
            cfg.mcp_servers.push(server);
            println!("  ✓ 已添加 MCP 服务器");
        } else {
            println!("  ⚠ 格式无效，期望: name|command|arg1 arg2|KEY=VAL");
        }
    }

    // ── Save ──
    let needs_api_key = cfg.provider.as_str() != "ollama";
    if needs_api_key && cfg.api_key.is_empty() {
        anyhow::bail!("{} 需要 API Key，配置未保存", cfg.provider);
    }

    cfg.save()?;
    let total_tools = crate::tools::TOOL_INDEX.len();
    let enabled_count = if cfg.enabled_tools.is_empty() {
        total_tools
    } else {
        cfg.enabled_tools.len()
    };

    println!("\n配置摘要：");
    println!("  Provider: {}", cfg.provider);
    println!(
        "  API Key: {}...{}",
        &cfg.api_key[..4.min(cfg.api_key.len())],
        &cfg.api_key[cfg.api_key.len().saturating_sub(4)..]
    );
    println!("  Base URL: {}", cfg.base_url);
    println!("  Model: {}", cfg.model);
    println!(
        "  搜索: {}",
        cfg.search_base_url.as_deref().unwrap_or("DuckDuckGo (free)")
    );
    println!("  MCP 服务器: {} 个", cfg.mcp_servers.len());
    println!(
        "  工具: {} ({} 个 / 总 {} 个)",
        if cfg.enabled_tools.is_empty() {
            "全部启用"
        } else {
            "部分启用"
        },
        enabled_count,
        total_tools,
    );
    println!("  运行 `i-rs-claw tools` 管理工具开关");

    Ok(())
}

// =============================================
// Tools subcommand
// =============================================

pub fn run_tools() -> anyhow::Result<()> {
    let mut cfg = Config::load()?;

    // If no explicit tool selection exists (empty = all enabled in runtime),
    // seed the UI with DEFAULT_TOOLS so only those 10 show as checked initially
    if cfg.enabled_tools.is_empty() {
        cfg.enabled_tools = crate::config::DEFAULT_TOOLS
            .iter()
            .map(|s| s.to_string())
            .collect();
    }

    let all_tools: Vec<&str> = crate::tools::TOOL_INDEX
        .iter()
        .map(|(n, _, _)| *n)
        .collect();
    let total = all_tools.len();

    // ── TUI setup ──
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut selection: usize = 0;
    let mut dirty = false;

    let result: anyhow::Result<()> = (|| {
        loop {
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
                    format!(
                        " ✦ 工具管理  [{}✓ / {}总]  ↑↓选择  Space切换  a全选  n清空  Enter保存  Esc取消",
                        checked_count, total
                    ),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ));
                f.render_widget(title, chunks[0]);

                // Tool list
                let items: Vec<ListItem> = all_tools
                    .iter()
                    .map(|name| {
                        let checked = cfg.enabled_tools.is_empty()
                            || cfg.enabled_tools.contains(*name);
                        let checkbox = if checked { "[✓]" } else { "[ ]" };
                        let desc = crate::tools::TOOL_INDEX
                            .iter()
                            .find(|(n, _, _)| n == name)
                            .map(|(_, d, _)| *d)
                            .unwrap_or("");
                        let text = format!(" {} {}", checkbox, name);
                        let line = Line::from(vec![
                            Span::styled(
                                text,
                                Style::default().fg(if checked {
                                    Color::Green
                                } else {
                                    Color::DarkGray
                                })
                                .add_modifier(if checked {
                                    Modifier::BOLD
                                } else {
                                    Modifier::empty()
                                }),
                            ),
                            Span::styled(
                                format!("  — {}", desc),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]);
                        ListItem::new(line)
                    })
                    .collect();

                let list = List::new(items)
                    .highlight_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                    )
                    .block(Block::default().borders(Borders::NONE));

                f.render_stateful_widget(
                    list,
                    chunks[1],
                    &mut ratatui::widgets::ListState::default()
                        .with_selected(Some(selection)),
                );
            })?;

            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Up => selection = selection.saturating_sub(1),
                    KeyCode::Down if selection + 1 < total => selection += 1,
                    KeyCode::Char(' ') => {
                        dirty = true;
                        let name = all_tools[selection];
                        if cfg.enabled_tools.is_empty() {
                            cfg.enabled_tools =
                                all_tools.iter().map(|s| s.to_string()).collect();
                        }
                        if cfg.enabled_tools.contains(name) {
                            cfg.enabled_tools.remove(name);
                        } else {
                            cfg.enabled_tools.insert(name.to_string());
                        }
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        dirty = true;
                        cfg.enabled_tools =
                            all_tools.iter().map(|s| s.to_string()).collect();
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        dirty = true;
                        cfg.enabled_tools.clear();
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
            println!(
                "✓ 已启用 {} 个工具 (停用 {} 个)",
                cfg.enabled_tools.len(),
                total - cfg.enabled_tools.len()
            );
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

pub fn run_session_list() -> anyhow::Result<()> {
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
// Session Export
// =============================================

pub fn run_export(session_id: &str, format: &str) -> anyhow::Result<()> {
    let session_mgr = crate::session::SessionManager::new(claw_dir().join("claw"));

    let output = match format {
        "md" => session_mgr.export_markdown(session_id),
        "json" => session_mgr.export_json(session_id),
        _ => None,
    };

    match output {
        Some(content) => {
            println!("{}", content);
            Ok(())
        }
        None => anyhow::bail!("未找到会话: {}", session_id),
    }
}

// =============================================
// Quick Ask (non-interactive)
// =============================================

pub fn run_ask(message: &str, _session_id: Option<&str>) -> anyhow::Result<()> {
    let config = crate::config::Config::load()?;
    let provider = crate::provider::create_provider(&config);

    let msgs = vec![
        serde_json::json!({
            "role": "system",
            "content": "你是一个有用的AI助手。请用中文简洁回答用户的问题。"
        }),
        serde_json::json!({
            "role": "user",
            "content": message
        }),
    ];

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            let _ = provider
                .stream_chat(&msgs, &[], &tx)
                .await;
        });

        use std::io::Write;
        while let Some(event) = rx.recv().await {
            match event {
                crate::llm::LlmEvent::Token(t) => {
                    print!("{}", t);
                    let _ = io::stdout().flush();
                }
                crate::llm::LlmEvent::Error(e) => {
                    eprintln!("\n错误: {}", e);
                    break;
                }
                _ => {}
            }
        }
        println!();
        Ok::<_, anyhow::Error>(())
    })?;

    Ok(())
}

// =============================================
// Gateway subcommand
// =============================================

pub fn run_gateway() -> anyhow::Result<()> {
    let config = crate::config::Config::load()?;
    let rt = tokio::runtime::Runtime::new()?;

    let core = std::sync::Arc::new(std::sync::Mutex::new(
        crate::core::AppCore::new(config.clone()),
    ));

    // Initialize MCP if configured
    if !config.mcp_servers.is_empty() {
        crate::core::engine::init_mcp(&config.mcp_servers);
    }

    #[allow(unused_mut)]
    let mut server = crate::gateway::GatewayServer::new();

    // Register platform adapters based on config
    if config.gateway.enabled {
        #[cfg(feature = "gateway-telegram")]
        if let Some(ref tg) = config.gateway.telegram {
            if tg.enabled {
                if let Some(ref token) = tg.token {
                    let adapter = crate::gateway::telegram::TelegramAdapter::new(
                        crate::gateway::telegram::TelegramConfig {
                            bot_token: token.clone(),
                            enabled: true,
                        },
                    );
                    server.register(Box::new(adapter));
                    println!("  ✓ Telegram bot registered");
                }
            }
        }

        #[cfg(feature = "gateway-wechat")]
        if let Some(ref wc) = config.gateway.wechat {
            if wc.enabled {
                let adapter = crate::gateway::wechat::WeChatAdapter::new(
                    crate::gateway::wechat::WeChatConfig { enabled: true },
                );
                server.register(Box::new(adapter));
                println!("  ✓ WeChat bot registered");
            }
        }
    }

    if server.adapter_count() == 0 {
        println!("ℹ No gateway adapters enabled. Configure them in config.toml:");
        println!("  [gateway]");
        println!("  enabled = true");
        println!("");
        println!("  [gateway.telegram]");
        println!("  enabled = true");
        println!("  token = \"your-bot-token\"");
        println!("");
        println!("  [gateway.wechat]");
        println!("  enabled = true");
        println!("  # Credentials obtained via QR login on first run");
        return Ok(());
    }

    println!("Gateway server running. Press Ctrl+C to stop.");
    rt.block_on(server.run(core));
    Ok(())
}

// =============================================
// Dashboard subcommand
// =============================================

#[cfg(feature = "dashboard")]
pub fn run_dashboard() -> anyhow::Result<()> {
    let config = crate::config::Config::load()?;
    let rt = tokio::runtime::Runtime::new()?;

    let core = std::sync::Arc::new(crate::core::AppCore::new(config.clone()));

    // Initialize MCP if configured
    if !core.config.mcp_servers.is_empty() {
        crate::core::engine::init_mcp(&core.config.mcp_servers);
    }

    // Discover plugins and merge into MCP config
    if core.config.plugins_auto_discover {
        let plugin_mgr = crate::plugin::PluginManager::new();
        let plugin_configs = plugin_mgr.to_mcp_configs();
        if !plugin_configs.is_empty() {
            // Plugin configs are already merged into mcp_servers at init
            eprintln!("  {} plugins discovered", plugin_configs.len(),);
        }
    }

    let dashboard = crate::dashboard::Dashboard::new(config.dashboard);
    println!(
        "{}  {}\n",
        " 🔷 i-rs-claw Dashboard".bold().bright_blue(),
        "🚀 Server starting...".bright_green()
    );
    rt.block_on(dashboard.run(core));
    Ok(())
}

#[cfg(not(feature = "dashboard"))]
pub fn run_dashboard() -> anyhow::Result<()> {
    anyhow::bail!(
        "Dashboard feature is not enabled. Rebuild with: cargo build --features dashboard"
    );
}

// =============================================
// Plugin subcommand
// =============================================

pub fn run_plugin_list() -> anyhow::Result<()> {
    let mgr = crate::plugin::PluginManager::new();

    if mgr.plugin_count() == 0 {
        println!("No plugins found in {:?}", mgr.plugins_dir());
        return Ok(());
    }

    println!("Plugins ({} discovered, {} enabled):\n", mgr.plugin_count(), mgr.enabled_count());
    for m in &mgr.manifests {
        let status = if mgr.is_enabled(&m.plugin.name) {
            "enabled"
        } else {
            "disabled"
        };
        println!(
            "  {:<20} v{:<8} [{}]  {}",
            m.plugin.name,
            m.plugin.version,
            status,
            m.plugin.description
        );
    }
    Ok(())
}

pub fn run_plugin_info(name: &str) -> anyhow::Result<()> {
    let mgr = crate::plugin::PluginManager::new();

    match mgr.find(name) {
        Some(m) => {
            let status = if mgr.is_enabled(name) {
                "enabled"
            } else {
                "disabled"
            };
            println!("Plugin: {}", m.plugin.name);
            println!("  Version:     {}", m.plugin.version);
            println!("  Description: {}", m.plugin.description);
            if let Some(ref author) = m.plugin.author {
                println!("  Author:      {}", author);
            }
            if let Some(ref url) = m.plugin.homepage {
                println!("  Homepage:    {}", url);
            }
            println!("  Status:      {}", status);
            println!("  Transport:   {}", m.transport.transport_type);
            if let Some(ref cmd) = m.transport.command {
                println!("  Command:     {}", cmd);
            }
            if let Some(ref url) = m.transport.url {
                println!("  URL:         {}", url);
            }
        }
        None => {
            println!("Plugin '{}' not found", name);
        }
    }
    Ok(())
}

pub fn run_plugin_enable(name: &str) -> anyhow::Result<()> {
    let mut mgr = crate::plugin::PluginManager::new();

    if mgr.find(name).is_none() {
        println!("Plugin '{}' not found", name);
        return Ok(());
    }

    mgr.enable(name);
    println!("Enabled plugin: {}", name);
    Ok(())
}

pub fn run_plugin_disable(name: &str) -> anyhow::Result<()> {
    let mut mgr = crate::plugin::PluginManager::new();

    if mgr.find(name).is_none() {
        println!("Plugin '{}' not found", name);
        return Ok(());
    }

    mgr.disable(name);
    println!("Disabled plugin: {}", name);
    Ok(())
}

// =============================================
// Helpers
// =============================================

fn claw_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("无法获取用户主目录")
        .join(".i-rs-claw")
}

/// Parse MCP server config from user input.
/// Format: name|command|arg1 arg2|KEY=VAL
fn parse_mcp_server(input: &str) -> Option<crate::mcp::McpServerConfig> {
    let parts: Vec<&str> = input.splitn(4, '|').collect();
    if parts.len() < 2 {
        return None;
    }
    let name = parts[0].trim().to_string();
    let command = parts[1].trim().to_string();
    if name.is_empty() || command.is_empty() {
        return None;
    }
    let args = parts.get(2).map(|s| s.trim().split_whitespace().map(|a| a.to_string()).collect());
    let env = parts.get(3).map(|s| s.trim().split_whitespace().map(|e| e.to_string()).collect());
    Some(crate::mcp::McpServerConfig {
        name,
        transport_type: "stdio".to_string(),
        command: Some(command),
        args,
        url: None,
        env,
    })
}
