mod app;
mod cli;
mod completion;
mod config;
mod gateway;
#[cfg(feature = "dashboard")]
mod server;
#[cfg(test)]
mod test_helpers;
mod theme;
mod tui;
mod ui;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "i-rs-claw",
    version,
    about = "AI personal assistant — Web Dashboard + TUI terminal interface"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start HTTP API server + Web Dashboard (default mode)
    Serve {
        /// Host to bind (default: 127.0.0.1; pass 0.0.0.0 to expose publicly)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to listen on (default: 3000)
        #[arg(long, short, default_value = "3000")]
        port: u16,
        /// Disable the Web Dashboard UI (API only)
        #[arg(long)]
        api_only: bool,
        /// Allow tools classified as High-risk to run without confirmation.
        /// DANGEROUS: only enable in sandboxed/CI environments.
        #[arg(long = "auto-approve")]
        auto_approve_high_risk: bool,
    },
    /// Start TUI terminal interface (debug/power-user mode)
    Tui {
        /// Resume a specific session by ID
        #[arg(long)]
        session: Option<String>,
        /// User ID for multi-tenant mode
        #[arg(long)]
        user: Option<String>,
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
        /// Export session as Markdown (provide session ID)
        #[arg(long)]
        export_md: Option<String>,
        /// Export session as JSON (provide session ID)
        #[arg(long)]
        export_json: Option<String>,
    },
    /// Send a message and print response (non-interactive)
    Ask {
        /// The message to send
        message: String,
        /// Session ID for context continuity (optional)
        #[arg(long)]
        session: Option<String>,
    },
    /// Start the gateway server for social platform integration
    Gateway,
    /// Start the dashboard web server (alias for `serve`)
    Dashboard,
    /// List and manage plugins
    Plugin {
        /// List all discovered plugins
        #[arg(long)]
        list: bool,
        /// Show detailed info for a plugin
        #[arg(long)]
        info: Option<String>,
        /// Enable a plugin
        #[arg(long)]
        enable: Option<String>,
        /// Disable a plugin
        #[arg(long)]
        disable: Option<String>,
    },
    /// List and manage skills
    Skill {
        /// List all installed skills
        #[arg(long)]
        list: bool,
        /// Create a new skill from template
        #[arg(long)]
        install: Option<String>,
        /// Remove a skill
        #[arg(long)]
        remove: Option<String>,
        /// Show skill details
        #[arg(long)]
        info: Option<String>,
    },
    /// Show token usage statistics
    Stats {
        /// Time period: today (default), 7d, 30d, all
        #[arg(long, default_value = "today")]
        period: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List and manage MCP servers
    Mcp {
        /// List all configured MCP servers
        #[arg(long)]
        list: bool,
        /// Enable a MCP server by name
        #[arg(long)]
        enable: Option<String>,
        /// Disable a MCP server by name
        #[arg(long)]
        disable: Option<String>,
        /// Test MCP connection and show discovered tools
        #[arg(long)]
        check: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let use_json_log = std::env::var("CLAW_LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    let trace_dir = std::env::var("CLAW_TRACE_DIR").ok();
    let trace_path = trace_dir.as_ref().map(|dir| {
        std::path::PathBuf::from(dir).join(format!(
            "trace-{}.jsonl",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        ))
    });

    if let Some(ref path) = trace_path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let subscriber = tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::builder()
                        .with_default_directive(tracing::Level::WARN.into())
                        .from_env_lossy(),
                )
                .json()
                .with_writer(std::sync::Arc::new(file));
            subscriber.init();
        } else {
            init_default_subscriber(use_json_log);
        }
    } else {
        init_default_subscriber(use_json_log);
    }

    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Serve {
        host: "127.0.0.1".to_string(),
        port: 3000,
        api_only: false,
        auto_approve_high_risk: false,
    }) {
        Command::Serve { host, port, api_only, auto_approve_high_risk } => {
            cli::run_serve(host, port, api_only, auto_approve_high_risk)
        }
        Command::Tui { session, user: _ } => tui::run(session.as_deref()),
        Command::Config => cli::run_config(),
        Command::Tools => cli::run_tools(),
        Command::Session { list: true, .. } => cli::run_session_list(),
        Command::Session { export_md: Some(id), .. } => cli::run_export(&id, "md"),
        Command::Session { export_json: Some(id), .. } => cli::run_export(&id, "json"),
        Command::Session { .. } => cli::run_session_list(),
        Command::Ask { message, session } => cli::run_ask(&message, session.as_deref()),
        Command::Gateway => cli::run_gateway(),
        Command::Dashboard => cli::run_dashboard(),
        Command::Plugin { list: true, .. } => cli::run_plugin_list(),
        Command::Plugin { info: Some(name), .. } => cli::run_plugin_info(&name),
        Command::Plugin { enable: Some(name), .. } => cli::run_plugin_enable(&name),
        Command::Plugin { disable: Some(name), .. } => cli::run_plugin_disable(&name),
        Command::Plugin { .. } => cli::run_plugin_list(),
        Command::Skill { list: true, .. } => cli::run_skill_list(),
        Command::Skill { install: Some(name), .. } => cli::run_skill_install(&name),
        Command::Skill { remove: Some(name), .. } => cli::run_skill_remove(&name),
        Command::Skill { info: Some(name), .. } => cli::run_skill_info(&name),
        Command::Skill { .. } => cli::run_skill_list(),
        Command::Stats { period, json } => cli::run_stats(&period, json),
        Command::Mcp { list: true, .. } => cli::run_mcp_list(),
        Command::Mcp { enable: Some(name), .. } => cli::run_mcp_enable(&name),
        Command::Mcp { disable: Some(name), .. } => cli::run_mcp_disable(&name),
        Command::Mcp { check: Some(name), .. } => cli::run_mcp_check(&name),
        Command::Mcp { .. } => cli::run_mcp_list(),
    }
}

fn init_default_subscriber(json: bool) {
    if json {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(tracing::Level::WARN.into())
                    .from_env_lossy(),
            )
            .json()
            .with_writer(std::io::stderr)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(tracing::Level::WARN.into())
                    .from_env_lossy(),
            )
            .with_writer(std::io::stderr)
            .init();
    }
}
