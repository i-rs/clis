use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_listen,
    handle_skill, handle_stats, handle_update,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-podcast")]
#[command(about = "Podcast and course tracking CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new entry
    Add {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        author: Option<String>,
        #[arg(short = 'd', long)]
        duration: Option<i64>,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(long)]
        remark: Vec<String>,
        #[arg(long)]
        notes: Vec<String>,
    },
    /// Delete an entry
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Get an entry by id
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// List all entries
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(short, long)]
        tag: Option<String>,
    },
    /// Record a listening entry
    Listen {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short = 'p', long)]
        position: i64,
        #[arg(long)]
        notes: Option<Vec<String>>,
    },
    /// Update an entry
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        author: Option<String>,
        #[arg(short = 'd', long)]
        duration: Option<i64>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(long)]
        remark: Option<Vec<String>>,
        #[arg(long)]
        notes: Option<Vec<String>>,
    },
    /// Show statistics
    Stats {},
    /// Show usage examples
    Example {},
    #[clap(subcommand)]
    Skill(commands::skill::SkillCommand),
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
            name,
            author,
            duration,
            tag,
            remark,
            notes,
        } => {
            handle_add(name, author, duration, tag, remark, notes, format)?;
        }
        Commands::Delete { name } => {
            handle_delete(name, format)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
        }
        Commands::List { status, tag } => {
            handle_list(status, tag, format)?;
        }
        Commands::Listen {
            name,
            position,
            notes,
        } => {
            handle_listen(name, position, notes, format)?;
        }
        Commands::Update {
            name,
            author,
            duration,
            tag,
            remark,
            notes,
        } => {
            handle_update(name, author, duration, tag, remark, notes, format)?;
        }
        Commands::Stats {} => {
            handle_stats(format)?;
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
