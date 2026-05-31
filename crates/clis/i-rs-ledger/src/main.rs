use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_update,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-ledger")]
#[command(about = "Accounting ledger CLI - track income, expenses, and transfers", long_about = None)]
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
        #[arg(value_name = "AMOUNT")]
        amount: f64,
        #[arg(value_name = "CURRENCY")]
        currency: String,
        #[arg(value_name = "TYPE")]
        entry_type: String,
        #[arg(value_name = "CATEGORY")]
        category: String,
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
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Update an entry
    Update {
        #[arg(value_name = "ID")]
        id: String,
        #[arg(short, long)]
        date: Option<String>,
        #[arg(short, long)]
        amount: Option<f64>,
        #[arg(short, long)]
        entry_type: Option<String>,
        #[arg(short, long)]
        category: Option<String>,
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
            date,
            amount,
            currency,
            entry_type,
            category,
            tag,
            remark,
        } => {
            handle_add(date, amount, currency, entry_type, category, tag, remark)?;
        }
        Commands::Delete { id } => {
            handle_delete(id)?;
        }
        Commands::List { category } => {
            handle_list(category, format)?;
        }
        Commands::Update {
            id,
            date,
            amount,
            entry_type,
            category,
            tag,
            remark,
        } => {
            handle_update(id, date, amount, entry_type, category, tag, remark)?;
        }
        Commands::Get { id } => {
            handle_get(id, format)?;
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
