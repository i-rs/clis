mod app;
mod cli;
mod config;
mod llm;
mod memory;
mod session;
mod skill_store;
mod tool_cache;
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
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Tui { session: None }) {
        Command::Tui { session } => tui::run(session.as_deref()),
        Command::Config => cli::run_config(),
        Command::Tools => cli::run_tools(),
        Command::Session { .. } => cli::run_session_list(),
    }
}
