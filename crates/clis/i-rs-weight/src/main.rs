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
#[command(name = "i-rs-weight")]
#[command(about = "Weight tracking CLI", long_about = None)]
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
        #[arg(value_name = "WEIGHT")]
        weight: f64,
        #[arg(short, long)]
        date: Option<String>,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    /// Delete an entry
    Delete {
        #[arg(value_name = "ID")]
        id: String,
    },
    /// List all entries
    List {
        #[arg(short = 'd', long)]
        days: Option<usize>,
        #[arg(short = 'c', long)]
        chart: bool,
        #[arg(short = 's', long)]
        stats: bool,
    },
    /// Update an entry
    Update {
        #[arg(value_name = "ID")]
        id: String,
        #[arg(short = 'w', long)]
        weight: Option<f64>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
    },
    /// Get an entry by id
    Get {
        #[arg(value_name = "ID")]
        id: String,
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
            weight,
            date,
            tag,
            remark,
        } => {
            handle_add(date, weight, tag, remark, format)?;
        }
        Commands::Delete { id } => {
            handle_delete(id, format)?;
        }
        Commands::Get { id } => {
            handle_get(id, format)?;
        }
        Commands::List { days, chart, stats } => {
            handle_list(days, chart, stats, format)?;
        }
        Commands::Update {
            id,
            weight,
            tag,
            remark,
        } => {
            handle_update(id, weight, tag, remark, format)?;
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
