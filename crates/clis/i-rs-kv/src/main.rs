use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_copy, handle_delete, handle_example, handle_get, handle_list, handle_rename,
    handle_search, handle_skill, handle_stats, handle_update,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod service;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-kv")]
#[command(about = "Key-Value storage CLI - store and retrieve simple key-value data", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new key-value entry
    Add {
        #[arg(value_name = "KEY")]
        key: String,
        #[arg(value_name = "VALUE")]
        value: String,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    /// Delete a key-value entry
    Delete {
        #[arg(value_name = "KEY")]
        key: String,
    },
    /// List all entries (with optional tag/pattern filtering)
    List {
        #[arg(short, long)]
        tag: Option<String>,
        #[arg(short, long)]
        pattern: Option<String>,
    },
    /// Update a key-value entry (value, tags, or remarks)
    Update {
        #[arg(value_name = "KEY")]
        key: String,
        #[arg(short, long)]
        value: Option<String>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
    },
    /// Get a value by key
    Get {
        #[arg(value_name = "KEY")]
        key: String,
    },
    /// Search entries by value or key pattern
    Search {
        #[arg(value_name = "QUERY")]
        query: String,
    },
    /// Show storage statistics (total entries, tags, etc.)
    Stats {},
    /// Copy an entry to a new key
    Copy {
        #[arg(value_name = "SRC_KEY")]
        src: String,
        #[arg(value_name = "DST_KEY")]
        dst: String,
    },
    /// Rename an entry key
    Rename {
        #[arg(value_name = "OLD_KEY")]
        old: String,
        #[arg(value_name = "NEW_KEY")]
        new: String,
    },
    /// Show usage examples
    Example {},
    /// AI skill system commands (info, teach, search, install, etc.)
    #[clap(subcommand)]
    Skill(commands::skill::SkillCommand),
    /// Data management commands (export, import, clear)
    #[clap(subcommand)]
    Data(commands::data::DataCommand),
}

fn main() {
    let cli = Cli::parse();
    let format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Table
    };

    i_rs_core::exit_on_error!(run(cli.command, format), cli.json);
}

fn run(command: Commands, format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Add {
            key,
            value,
            tag,
            remark,
        } => {
            handle_add(key, value, tag, remark, format)?;
        }
        Commands::Delete { key } => {
            handle_delete(key, format)?;
        }
        Commands::List { tag, pattern } => {
            handle_list(tag, pattern, format)?;
        }
        Commands::Update {
            key,
            value,
            tag,
            remark,
        } => {
            handle_update(key, value, tag, remark, format)?;
        }
        Commands::Get { key } => {
            handle_get(key, format)?;
        }
        Commands::Search { query } => {
            handle_search(query, format)?;
        }
        Commands::Stats {} => {
            handle_stats(format)?;
        }
        Commands::Copy { src, dst } => {
            handle_copy(src, dst, format)?;
        }
        Commands::Rename { old, new } => {
            handle_rename(old, new, format)?;
        }
        Commands::Example {} => {
            handle_example();
        }
        Commands::Skill(cmd) => {
            handle_skill(&cmd)?;
        }
        Commands::Data(commands) => commands::data::handle(&commands)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests;
