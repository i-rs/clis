mod config_wizard;
mod tools_ui;

pub(crate) use config_wizard::claw_dir;
pub use config_wizard::run_config;
pub use tools_ui::run_tools;

use i_rs_claw_core::config::Config;
use i_rs_claw_core::session::SessionManager;
#[cfg(feature = "dashboard")]
use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::sync::Arc;

fn skill_store() -> i_rs_claw_core::skill_store::SkillStore {
    let claw_dir = i_rs_claw_core::utils::claw_dir().expect("无法获取用户主目录");
    i_rs_claw_core::skill_store::SkillStore::for_agent(&claw_dir, "default")
}

// =============================================
// Session subcommand
// =============================================

pub fn run_session_list() -> anyhow::Result<()> {
    let session_mgr = SessionManager::new(claw_dir())?;

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
    let session_mgr = i_rs_claw_core::session::SessionManager::new(claw_dir())?;

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
    let config = i_rs_claw_core::config::Config::load()?;
    let client = i_rs_claw_core::providers::shared_client();
    let provider = i_rs_claw_core::providers::create_provider(&client, &config);

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
        let tid = uuid::Uuid::new_v4().to_string();

        tokio::spawn(async move {
            let _ = provider.stream_chat(&msgs, &[], &tx, &tid).await;
        });

        while let Some(event) = rx.recv().await {
            match event {
                i_rs_claw_core::llm::LlmEvent::Token(t) => {
                    print!("{}", t);
                    let _ = io::stdout().flush();
                }
                i_rs_claw_core::llm::LlmEvent::Error(e) => {
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
    let config = i_rs_claw_core::config::Config::load()?;
    let rt = tokio::runtime::Runtime::new()?;

    let core = std::sync::Arc::new(tokio::sync::RwLock::new(i_rs_claw_core::core::AppCore::new(
        config.clone(),
    )?));

    #[allow(unused_mut)]
    let mut server = crate::gateway::GatewayServer::new();

    // Register platform adapters based on config
    if config.gateway.enabled {
        if let Some(ref tg) = config.gateway.telegram
            && tg.enabled
            && let Some(ref token) = tg.token
        {
            let adapter = crate::gateway::telegram::TelegramAdapter::new(
                crate::gateway::telegram::TelegramConfig {
                    bot_token: token.clone(),
                    enabled: true,
                    agent_id: tg.agent_id.clone().unwrap_or_else(|| "default".to_string()),
                    allowed_users: tg.allowed_users.clone(),
                },
            );
            server.register(std::sync::Arc::new(adapter));
            println!("  ✓ Telegram bot registered");
        }

        if let Some(ref wc) = config.gateway.wechat
            && wc.enabled
        {
            let adapter =
                crate::gateway::wechat::WeChatAdapter::new(crate::gateway::wechat::WeChatConfig {
                    enabled: true,
                    agent_id: wc.agent_id.clone().unwrap_or_else(|| "default".to_string()),
                    allowed_users: wc.allowed_users.clone(),
                });
            server.register(std::sync::Arc::new(adapter));
            println!("  ✓ WeChat bot registered");
        }
    }

    if server.adapter_count() == 0 {
        println!("ℹ No gateway adapters enabled. Configure them in config.toml:");
        println!("  [gateway]");
        println!("  enabled = true");
        println!();
        println!("  [gateway.telegram]");
        println!("  enabled = true");
        println!("  token = \"your-bot-token\"");
        println!();
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
// Serve subcommand (API server + Web Dashboard)
// =============================================

#[cfg(feature = "dashboard")]
pub fn run_serve(
    cli_host: Option<String>,
    cli_port: Option<u16>,
    api_only: bool,
    auto_approve_high_risk: bool,
) -> anyhow::Result<()> {
    let mut config = i_rs_claw_core::config::Config::load()?;

    // CLI flags override config file values; fall back to config, then defaults.
    let host = cli_host.unwrap_or_else(|| config.dashboard.host.clone());
    let port = cli_port.unwrap_or(config.dashboard.port);

    // Warn on public bind — caller explicitly opted in.
    if host == "0.0.0.0" || host == "::" {
        tracing::warn!(
            host = %host,
            "SECURITY: claw serve is binding to a public address. \
             Anyone with the auth token can access the dashboard. \
             Use 127.0.0.1 (default) for local-only access."
        );
        eprintln!(
            "  {}  {}  host={} — dashboard will be reachable from the network",
            "⚠".yellow(),
            "SECURITY WARNING".bold().red(),
            host
        );
    }

    // Apply CLI flag to the runtime HitlPolicy. We do NOT persist this flag
    // to disk to avoid accidentally enabling it permanently.
    if auto_approve_high_risk {
        config.hitl.auto_approve_high_risk = true;
        tracing::warn!(
            "SECURITY: --auto-approve passed; high-risk tools will execute without confirmation"
        );
        eprintln!(
            "  {}  {}  high-risk tools will be auto-approved",
            "⚠".yellow(),
            "SECURITY WARNING".bold().red(),
        );
    }

    let rt = tokio::runtime::Runtime::new()?;

    let core = i_rs_claw_core::core::AppCore::new(config.clone())?;

    if core.config.plugins_auto_discover {
        let plugin_mgr = i_rs_claw_core::plugin::PluginManager::new();
        let plugin_configs = plugin_mgr.to_mcp_configs();
        if !plugin_configs.is_empty() {
            tracing::info!("{} plugins discovered", plugin_configs.len());
        }
    }

    println!(
        " {}  {}\n",
        " 🔷 i-rs-claw Serve".bold().bright_blue(),
        "🚀 Server starting...".bright_green()
    );
    rt.block_on(crate::server::run(core, host, port, api_only));
    Ok(())
}

#[cfg(not(feature = "dashboard"))]
pub fn run_serve(
    _host: Option<String>,
    _port: Option<u16>,
    _api_only: bool,
    _auto_approve_high_risk: bool,
) -> anyhow::Result<()> {
    anyhow::bail!(
        "Dashboard feature is not enabled. Rebuild with: cargo build --features dashboard"
    );
}

/// Legacy alias for `claw serve` (used by the `Dashboard` subcommand).
pub fn run_dashboard() -> anyhow::Result<()> {
    let config = i_rs_claw_core::config::Config::load()?;
    run_serve(
        Some(config.dashboard.host.clone()),
        Some(config.dashboard.port),
        false,
        config.hitl.auto_approve_high_risk,
    )
}

// =============================================
// Plugin subcommand
// =============================================

pub fn run_plugin_list() -> anyhow::Result<()> {
    let mgr = i_rs_claw_core::plugin::PluginManager::new();

    if mgr.plugin_count() == 0 {
        println!("No plugins found in {:?}", mgr.plugins_dir());
        return Ok(());
    }

    println!(
        "Plugins ({} discovered, {} enabled):\n",
        mgr.plugin_count(),
        mgr.enabled_count()
    );
    for m in &mgr.manifests {
        let status = if mgr.is_enabled(&m.plugin.name) {
            "enabled"
        } else {
            "disabled"
        };
        println!(
            "  {:<20} v{:<8} [{}]  {}",
            m.plugin.name, m.plugin.version, status, m.plugin.description
        );
    }
    Ok(())
}

pub fn run_plugin_info(name: &str) -> anyhow::Result<()> {
    let mgr = i_rs_claw_core::plugin::PluginManager::new();

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
    let mut mgr = i_rs_claw_core::plugin::PluginManager::new();

    if mgr.find(name).is_none() {
        println!("Plugin '{}' not found", name);
        return Ok(());
    }

    mgr.enable(name);
    println!("Enabled plugin: {}", name);
    Ok(())
}

pub fn run_plugin_disable(name: &str) -> anyhow::Result<()> {
    let mut mgr = i_rs_claw_core::plugin::PluginManager::new();

    if mgr.find(name).is_none() {
        println!("Plugin '{}' not found", name);
        return Ok(());
    }

    mgr.disable(name);
    println!("Disabled plugin: {}", name);
    Ok(())
}

// =============================================
// MCP subcommand
// =============================================

pub fn run_mcp_list() -> anyhow::Result<()> {
    let cfg = Config::load()?;

    // Also check for plugin-discovered MCP servers
    let mut all_mcp = cfg.mcp_servers.clone();
    if cfg.plugins_auto_discover {
        let plugin_mgr = i_rs_claw_core::plugin::PluginManager::new();
        for pc in plugin_mgr.to_mcp_configs() {
            if !all_mcp.iter().any(|s| s.name == pc.name) {
                all_mcp.push(pc);
            }
        }
    }

    if all_mcp.is_empty() {
        println!("未配置 MCP 服务器，也没有发现插件");
        return Ok(());
    }

    let config_count = cfg.mcp_servers.len();
    let plugin_count = all_mcp.len() - config_count;
    println!("MCP 服务器 (配置 {} 个", config_count);
    if plugin_count > 0 {
        print!(" + 插件 {} 个", plugin_count);
    }
    println!("):\n");

    for srv in &all_mcp {
        let source = if srv.name.starts_with("plugin:") {
            "插件"
        } else {
            "配置"
        };
        let status = if srv.enabled { "enabled" } else { "disabled" };
        let cmd = srv.command.as_deref().unwrap_or("-");
        println!(
            "  {:<20} [{:<6}] [{:<8}]  {}",
            srv.name, source, status, cmd,
        );
    }
    Ok(())
}

pub fn run_mcp_check(name: &str) -> anyhow::Result<()> {
    // Collect potential configs from config file and plugins
    let cfg = Config::load()?;
    let mut candidates: Vec<i_rs_claw_core::mcp::McpServerConfig> = cfg.mcp_servers.clone();

    if cfg.plugins_auto_discover {
        let plugin_mgr = i_rs_claw_core::plugin::PluginManager::new();
        for pc in plugin_mgr.to_mcp_configs() {
            if !candidates.iter().any(|s| s.name == pc.name) {
                candidates.push(pc);
            }
        }
    }

    // Find by exact name or plugin:name prefix
    let srv = candidates.iter().find(|s| s.name == name).or_else(|| {
        candidates.iter().find(|s| {
            let stripped = s.name.strip_prefix("plugin:").unwrap_or(&s.name);
            stripped == name
        })
    });

    match srv {
        Some(server) => {
            println!("正在连接 MCP 服务器 '{}'...", server.name);
            println!("  传输: {}", server.transport_type);
            if let Some(cmd) = &server.command {
                println!("  命令: {}", cmd);
            }
            if let Some(ref args) = server.args {
                println!("  参数: {:?}", args);
            }

            // Try to connect
            let rt = Arc::new(tokio::runtime::Runtime::new()?);
            match i_rs_claw_core::mcp::McpClient::connect(server, &rt) {
                Ok(client) => {
                    match client.initialize() {
                        Ok(()) => println!("  ✓ 初始化成功"),
                        Err(e) => {
                            println!("  ✗ 初始化失败: {}", e);
                            return Ok(());
                        }
                    }
                    match client.list_tools() {
                        Ok(tools) => {
                            println!("  ✓ 发现 {} 个工具:", tools.len());
                            for t in &tools {
                                println!("    - {}: {}", t.name, t.description);
                            }
                        }
                        Err(e) => println!("  ✗ 工具发现失败: {}", e),
                    }
                }
                Err(e) => {
                    println!("  ✗ 连接失败: {}", e);
                }
            }
        }
        None => {
            println!("未找到 MCP 服务器 '{}'", name);
            println!("可用服务器:");
            for c in &candidates {
                let stripped = c.name.strip_prefix("plugin:").unwrap_or(&c.name);
                println!("  - {} ({})", stripped, c.name);
            }
        }
    }
    Ok(())
}

pub fn run_mcp_enable(name: &str) -> anyhow::Result<()> {
    let mut cfg = Config::load()?;

    match cfg.mcp_servers.iter_mut().find(|s| s.name == name) {
        Some(srv) => {
            srv.enabled = true;
            cfg.save()?;
            println!("Enabled MCP server: {}", name);
        }
        None => {
            println!("MCP server '{}' not found", name);
        }
    }
    Ok(())
}

pub fn run_mcp_disable(name: &str) -> anyhow::Result<()> {
    let mut cfg = Config::load()?;

    match cfg.mcp_servers.iter_mut().find(|s| s.name == name) {
        Some(srv) => {
            srv.enabled = false;
            cfg.save()?;
            println!("Disabled MCP server: {}", name);
        }
        None => {
            println!("MCP server '{}' not found", name);
        }
    }
    Ok(())
}

// =============================================
// Skill subcommand
// =============================================

pub fn run_skill_list() -> anyhow::Result<()> {
    let store = skill_store();
    let skills = store.list_skills();
    if skills.is_empty() {
        println!("暂无安装的技能");
        return Ok(());
    }
    println!("已安装的技能 ({} 个):\n", skills.len());
    for s in &skills {
        // Parse frontmatter to show description
        let desc = i_rs_claw_core::skill_store::parse_frontmatter(&s.content)
            .0
            .and_then(|t| {
                t.get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();
        if !desc.is_empty() {
            println!("  {:<20} — {}", s.name, desc);
        } else {
            println!("  {:<20}", s.name);
        }
    }
    Ok(())
}

pub fn run_skill_install(name: &str) -> anyhow::Result<()> {
    let store = skill_store();
    let template = i_rs_claw_core::skill_store::SkillStore::skill_template(name);
    store.install(name, &template)?;
    println!("✓ 已创建技能 '{}'", name);
    println!(
        "  编辑文件: {:?}",
        store.path().join(format!("{}.md", name))
    );
    Ok(())
}

pub fn run_skill_remove(name: &str) -> anyhow::Result<()> {
    let store = skill_store();
    if store.get_skill(name).is_none() {
        println!("技能 '{}' 未找到", name);
        return Ok(());
    }
    store.remove(name)?;
    println!("✓ 已删除技能 '{}'", name);
    Ok(())
}

pub fn run_skill_info(name: &str) -> anyhow::Result<()> {
    let store = skill_store();
    match store.get_skill(name) {
        Some(def) => {
            println!("技能: {}", def.name);
            println!("  描述: {}", def.description);
            if let Some(ref params) = def.parameters {
                println!(
                    "  参数: {}",
                    serde_json::to_string_pretty(params).unwrap_or_default()
                );
            } else {
                println!("  类型: 指令技能");
            }
            println!("  内容:");
            for line in def.content.lines() {
                println!("    {}", line);
            }
        }
        None => {
            println!("技能 '{}' 未找到", name);
        }
    }
    Ok(())
}

// =============================================
// Stats subcommand
// =============================================

pub fn run_stats(period: &str, json: bool) -> anyhow::Result<()> {
    let claw_data_dir = claw_dir();
    let cfg = i_rs_claw_core::config::Config::load()?;
    let stats_mgr = i_rs_claw_core::stats::StatsManager::new(&claw_data_dir, &cfg.stats, cfg.tz_offset);

    // Clean up expired records before querying
    if cfg.stats.enabled && cfg.stats.keep_days > 0 {
        stats_mgr.cleanup(cfg.stats.keep_days);
    }

    let stats_period = match period {
        "7d" | "7days" => i_rs_claw_core::stats::StatsPeriod::Last7Days,
        "30d" | "30days" => i_rs_claw_core::stats::StatsPeriod::Last30Days,
        "all" => i_rs_claw_core::stats::StatsPeriod::All,
        _ => i_rs_claw_core::stats::StatsPeriod::Today,
    };

    let result = stats_mgr.query(stats_period);

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    let period_label = match result.period {
        i_rs_claw_core::stats::StatsPeriod::Today => "今日",
        i_rs_claw_core::stats::StatsPeriod::Last7Days => "近 7 天",
        i_rs_claw_core::stats::StatsPeriod::Last30Days => "近 30 天",
        i_rs_claw_core::stats::StatsPeriod::All => "全部",
        i_rs_claw_core::stats::StatsPeriod::Custom { .. } => "自定义",
    };

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Token 用量统计 · {}", period_label);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  请求次数:     {:>8}", result.total_requests);
    println!("  输入 Token:   {:>8}", result.total_prompt_tokens);
    println!("  输出 Token:   {:>8}", result.total_completion_tokens);
    println!("  总 Token:     {:>8}", result.total_tokens);
    if result.total_cost_usd > 0.001 {
        println!("  预估费用:     ${:.4}", result.total_cost_usd);
    }
    println!("  平均延迟:     {:>8.1}ms", result.avg_latency_ms);
    println!(
        "  成功率:       {:>7}%",
        (result.success_rate * 100.0).round() / 100.0
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if !result.by_model.is_empty() {
        println!("\n按模型:");
        for m in &result.by_model {
            println!(
                "  {:<30} {:>4}次 {:>8} tok  ${:.4}",
                m.model, m.request_count, m.total_tokens, m.total_cost_usd
            );
        }
    }

    if !result.by_agent.is_empty() {
        println!("\n按 Agent:");
        for a in &result.by_agent {
            println!(
                "  {:<30} {:>4}次 {:>8} tok  ${:.4}",
                a.agent_id, a.request_count, a.total_tokens, a.total_cost_usd
            );
        }
    }

    Ok(())
}
