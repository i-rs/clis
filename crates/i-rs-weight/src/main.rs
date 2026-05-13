use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_list, handle_update};

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-weight")]
#[command(about = "Weight tracking CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "DATE")]
        date: String,
        #[arg(value_name = "WEIGHT")]
        weight: f64,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "DATE")]
        date: String,
    },
    List {
        #[arg(short = 'd', long)]
        days: Option<usize>,
        #[arg(short = 'c', long)]
        chart: bool,
        #[arg(short = 's', long)]
        stats: bool,
    },
    Update {
        #[arg(value_name = "DATE")]
        date: String,
        #[arg(short = 'w', long)]
        weight: Option<f64>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
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
        Commands::Add { date, weight, remark } => {
            handle_add(date, weight, remark)?;
        }
        Commands::Delete { date } => {
            handle_delete(date)?;
        }
        Commands::List { days, chart, stats } => {
            handle_list(days, chart, stats)?;
        }
        Commands::Update { date, weight, remark } => {
            handle_update(date, weight, remark)?;
        }
    }

    Ok(())
}
