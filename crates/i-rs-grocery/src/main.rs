use clap::{Parser, Subcommand};
use commands::{handle_add, handle_clear, handle_delete, handle_example, handle_get, handle_list, handle_purchase, handle_skill, handle_update, SkillCommand};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-grocery")]
#[command(about = "Grocery list CLI - manage your shopping list with quantities and purchase tracking", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "QUANTITY", default_value = "1")]
        quantity: i32,
        #[arg(value_name = "UNIT", default_value = "item")]
        unit: String,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Purchase {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Clear {},
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(short, long)]
        tag: Option<String>,
        #[arg(short, long)]
        purchased: bool,
        #[arg(short = 'n', long)]
        needed: bool,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short = 'q', long)]
        quantity: Option<i32>,
        #[arg(short = 'u', long)]
        unit: Option<String>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
    },
    Example {},
    Skill {
        #[arg(value_name = "SUB_COMMAND")]
        sub: Option<String>,
    },
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
        Commands::Add { name, quantity, unit, tag, remark } => {
            handle_add(name, quantity, unit, tag, remark)?;
        }
        Commands::Purchase { name } => {
            handle_purchase(name)?;
        }
        Commands::Clear {} => {
            handle_clear()?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { tag, purchased, needed } => {
            let status_filter = if purchased { Some(true) } else if needed { Some(false) } else { None };
            handle_list(tag, status_filter, format)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
        }
        Commands::Update { name, quantity, unit, tag, remark } => {
            handle_update(name, quantity, unit, tag, remark)?;
        }
        Commands::Example {} => {
            handle_example();
        }
        Commands::Skill { sub } => {
            let skill_cmd = match sub.as_deref() {
                Some("summary") => Some(SkillCommand::Summary),
                Some("content") => Some(SkillCommand::Content),
                Some("raw") => Some(SkillCommand::Raw),
                None => None,
                _ => {
                    eprintln!("Invalid subcommand. Use: summary, content, or raw");
                    std::process::exit(1);
                }
            };
            handle_skill(skill_cmd);
        }
        Commands::Data(commands) => { commands::data::handle(&commands)? }
    }
    Ok(())
}
