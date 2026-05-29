use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "i-rs-code", about = "Code editor AI agent", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true, help = "Show debug-level logs")]
    pub debug: bool,

    #[arg(short, long, global = true, help = "Show verbose output")]
    pub verbose: bool,
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
    /// Search conversation history
    Search {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Show version information
    Version,
    /// Manage sessions
    #[command(subcommand)]
    Sessions(SessionsCommands),
    /// Show or set workspace directory
    Workspace {
        #[arg(help = "If provided, sets the workspace directory")]
        path: Option<String>,
    },
    /// System diagnostics
    Doctor,
    /// Manage MCP servers
    #[command(subcommand)]
    Mcp(McpCommands),
    /// Manage custom tool plugins
    #[command(subcommand)]
    Plugins(PluginsCommands),
    /// Manage skill store
    #[command(subcommand)]
    Skill(SkillCommands),
}

#[derive(Subcommand, Debug)]
pub enum SessionsCommands {
    /// List all saved sessions
    List,
    /// Show session details
    Show {
        id: String,
        #[arg(long, help = "Show full message content")]
        full: bool,
    },
    /// Delete a session
    Delete {
        id: String,
    },
    /// Export session as markdown
    Export {
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// List configured MCP servers
    List,
    /// Add an MCP server
    Add {
        name: String,
        #[arg(long)]
        command: String,
        #[arg(long)]
        args: Option<Vec<String>>,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        env: Option<Vec<String>>,
    },
    /// Remove an MCP server
    Remove {
        name: String,
    },
    /// Connect and test an MCP server
    Test {
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum PluginsCommands {
    /// List installed plugins
    List,
    /// Show plugin directories
    Dir,
}

#[derive(Subcommand, Debug)]
pub enum SkillCommands {
    /// List installed skills
    List,
    /// Show a skill's content
    Get {
        name: String,
    },
    /// Create a new skill
    Create {
        name: String,
        #[arg(long, help = "Description of the skill")]
        description: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current config
    Show,
    /// Interactive setup wizard
    Init,
    /// Set a config value: provider|api_key|base_url|model|workspace
    Set {
        key: String,
        value: String,
    },
}
