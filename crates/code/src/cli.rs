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
    Tui,
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
    /// View/edit config
    Config,
}
