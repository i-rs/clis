use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill,
    handle_update,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod service;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-mood")]
#[command(about = "Mood tracking CLI", long_about = None)]
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
        #[arg(value_name = "DATE")]
        date: String,
        #[arg(value_name = "MOOD")]
        mood: String,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        content: Vec<String>,
    },
    /// Delete an entry
    Delete {
        #[arg(value_name = "DATE")]
        date: String,
    },
    /// List all entries
    List {
        #[arg(short = 'd', long)]
        days: Option<usize>,
        #[arg(short = 'c', long)]
        calendar: bool,
    },
    /// Update an entry
    Update {
        #[arg(value_name = "DATE")]
        date: String,
        #[arg(short = 'm', long)]
        mood: Option<String>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        content: Option<Vec<String>>,
    },
    /// Get an entry by id
    Get {
        #[arg(value_name = "DATE")]
        date: String,
    },
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
            date,
            mood,
            tag,
            content,
        } => {
            handle_add(date, mood, tag, content, format)?;
        }
        Commands::Delete { date } => {
            handle_delete(date, format)?;
        }
        Commands::List { days, calendar } => {
            handle_list(days, calendar, format)?;
        }
        Commands::Update {
            date,
            mood,
            tag,
            content,
        } => {
            handle_update(date, mood, tag, content, format)?;
        }
        Commands::Get { date } => {
            handle_get(date, format)?;
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
