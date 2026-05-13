use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_get, handle_list, handle_update};
use storage::init_keyring;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-password")]
#[command(about = "Password management CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "URL")]
        url: String,
        #[arg(short, long)]
        account: Option<String>,
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
        url: Option<String>,
        #[arg(short, long)]
        account: Option<String>,
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
            url,
            account,
            password,
            tag,
            remark,
        } => {
            handle_add(name, url, account, password, tag, remark)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { tag } => {
            handle_list(tag)?;
        }
        Commands::Update {
            name,
            url,
            account,
            password,
            tag,
            remark,
        } => {
            handle_update(name, url, account, password, tag, remark)?;
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
