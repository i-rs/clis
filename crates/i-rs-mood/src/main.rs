use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_list, handle_update};

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-mood")]
#[command(about = "Mood tracking CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
    Delete {
        #[arg(value_name = "DATE")]
        date: String,
    },
    List {
        #[arg(short = 'd', long)]
        days: Option<usize>,
        #[arg(short = 'c', long)]
        calendar: bool,
    },
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
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Add { date, mood, tag, content } => {
            handle_add(date, mood, tag, content)?;
        }
        Commands::Delete { date } => {
            handle_delete(date)?;
        }
        Commands::List { days, calendar } => {
            handle_list(days, calendar)?;
        }
        Commands::Update { date, mood, tag, content } => {
            handle_update(date, mood, tag, content)?;
        }
    }

    Ok(())
}
