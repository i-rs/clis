use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "i-rs-code", about = "Code editor AI agent", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Enter TUI full-screen mode
    Tui {
        #[arg(long, help = "Session ID to resume")]
        session: Option<String>,
    },
    /// One-shot conversation
    Chat {
        prompt: String,
        #[arg(long)]
        json: bool,
    },
    /// Agent mode for claw (JSON-RPC over stdio)
    Agent {
        #[arg(long)]
        task_id: String,
    },
    /// Manage configuration
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current config
    Show,
    /// Interactive setup wizard
    Init,
    /// Set a config value: provider|api_key|base_url|model
    Set {
        key: String,
        value: String,
    },
}
