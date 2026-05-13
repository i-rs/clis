use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_get, handle_list, handle_update};
use storage::init_keyring;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-domain")]
#[command(about = "Domain management CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "EXPIRY_DATE")]
        expiry_date: String,
        #[arg(short, long)]
        registrar: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        expiry_date: Option<String>,
        #[arg(short, long)]
        registrar: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short = 's', long)]
        show_password: bool,
    },
}

fn main() {
    init_keyring();

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
            expiry_date,
            registrar,
            password,
            tag,
            remark,
        } => {
            handle_add(name, expiry_date, registrar, password, tag, remark)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { tag } => {
            handle_list(tag)?;
        }
        Commands::Update {
            name,
            expiry_date,
            registrar,
            password,
            tag,
            remark,
        } => {
            handle_update(name, expiry_date, registrar, password, tag, remark)?;
        }
        Commands::Get {
            name,
            show_password,
        } => {
            handle_get(name, show_password)?;
        }
    }

    Ok(())
}
