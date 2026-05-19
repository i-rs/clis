mod app;
mod cli;
mod completion;
mod config;
mod convstore;
mod core;
#[cfg(feature = "dashboard")]
mod dashboard;
mod gateway;
mod llm;
mod mcp;
mod plugin;
mod memory;
mod provider;
mod router;
mod semantic;
mod session;
mod skill_store;
mod tool_cache;
mod theme;
mod tools;
mod tui;
mod ui;
mod utils;

use clap::{Parser, Subcommand};

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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Tui { session: None }) {
        Command::Tui { session } => tui::run(session.as_deref()),
        Command::Config => cli::run_config(),
        Command::Tools => cli::run_tools(),
        Command::Session {
            list: true,
            ..
        } => cli::run_session_list(),
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
        Command::Plugin {
            list: true,
            ..
        } => cli::run_plugin_list(),
        Command::Plugin {
            info: Some(name),
            ..
        } => cli::run_plugin_info(&name),
        Command::Plugin {
            enable: Some(name),
            ..
        } => cli::run_plugin_enable(&name),
        Command::Plugin {
            disable: Some(name),
            ..
        } => cli::run_plugin_disable(&name),
        Command::Plugin { .. } => cli::run_plugin_list(),
    }
}
