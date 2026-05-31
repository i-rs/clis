mod app;
mod cli;
mod completion;
mod config;
mod convstore;
mod core;
#[cfg(feature = "dashboard")]
mod dashboard;
mod error;
mod gateway;
mod llm;
mod mcp;
mod memory;
mod plugin;
mod providers;
mod semantic;
mod session;
mod skill_store;
mod stats;
mod storage;
mod theme;
mod tool_cache;
mod tools;
mod tui;
mod ui;
mod utils;

#[cfg(test)]
#[path = "test_helpers.rs"]
pub(crate) mod test_helpers;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "i-rs-claw",
    version,
    about = "TUI intelligent personal data assistant for i-rs CLI tools"
)]
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
    /// Start the dashboard web server
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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::WARN.into())
                .from_env_lossy(),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Tui { session: None }) {
        Command::Tui { session } => tui::run(session.as_deref()),
        Command::Config => cli::run_config(),
        Command::Tools => cli::run_tools(),
        Command::Session { list: true, .. } => cli::run_session_list(),
        Command::Session {
            export_md: Some(id),
            ..
        } => cli::run_export(&id, "md"),
        Command::Session {
            export_json: Some(id),
            ..
        } => cli::run_export(&id, "json"),
        Command::Session { .. } => cli::run_session_list(),
        Command::Ask { message, session } => cli::run_ask(&message, session.as_deref()),
        Command::Gateway => cli::run_gateway(),
        Command::Dashboard => cli::run_dashboard(),
        Command::Plugin { list: true, .. } => cli::run_plugin_list(),
        Command::Plugin {
            info: Some(name), ..
        } => cli::run_plugin_info(&name),
        Command::Plugin {
            enable: Some(name), ..
        } => cli::run_plugin_enable(&name),
        Command::Plugin {
            disable: Some(name),
            ..
        } => cli::run_plugin_disable(&name),
        Command::Plugin { .. } => cli::run_plugin_list(),
        Command::Skill { list: true, .. } => cli::run_skill_list(),
        Command::Skill {
            install: Some(name),
            ..
        } => cli::run_skill_install(&name),
        Command::Skill {
            remove: Some(name), ..
        } => cli::run_skill_remove(&name),
        Command::Skill {
            info: Some(name), ..
        } => cli::run_skill_info(&name),
        Command::Skill { .. } => cli::run_skill_list(),
        Command::Stats { period, json } => cli::run_stats(&period, json),
        Command::Mcp { list: true, .. } => cli::run_mcp_list(),
        Command::Mcp {
            enable: Some(name), ..
        } => cli::run_mcp_enable(&name),
        Command::Mcp {
            disable: Some(name),
            ..
        } => cli::run_mcp_disable(&name),
        Command::Mcp {
            check: Some(name), ..
        } => cli::run_mcp_check(&name),
        Command::Mcp { .. } => cli::run_mcp_list(),
    }
}
