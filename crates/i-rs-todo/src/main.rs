use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_done, handle_get, handle_list, handle_update};

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-todo")]
#[command(about = "Todo management CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short = 'p', long)]
        priority: Option<String>,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        content: Vec<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Done {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(short, long)]
        pending: bool,
        #[arg(short, long)]
        done: bool,
        #[arg(short = 't', long)]
        tag: Option<String>,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short = 'p', long)]
        priority: Option<String>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        content: Option<Vec<String>>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
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
        Commands::Add {
            name,
            title,
            priority,
            tag,
            content,
        } => {
            handle_add(name, title, priority, tag, content)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::Done { name } => {
            handle_done(name)?;
        }
        Commands::List {
            pending,
            done,
            tag,
        } => {
            handle_list(false, pending, done, tag)?;
        }
        Commands::Update {
            name,
            title,
            priority,
            tag,
            content,
        } => {
            handle_update(name, title, priority, tag, content)?;
        }
        Commands::Get { name } => {
            handle_get(name)?;
        }
    }

    Ok(())
}
