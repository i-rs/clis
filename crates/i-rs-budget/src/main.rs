#![allow(dead_code)]
use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_expense, handle_get, handle_list,
    handle_skill, handle_stats, handle_update, SkillCommand,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-budget")]
#[command(about = "Budget management CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "CATEGORY")]
        category: String,
        #[arg(value_name = "AMOUNT")]
        amount: f64,
        #[arg(short, long)]
        period: Option<String>,
        #[arg(short, long)]
        tags: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Expense {
        #[arg(value_name = "CATEGORY")]
        category: String,
        #[arg(value_name = "AMOUNT")]
        amount: f64,
        #[arg(short = 'd', long)]
        description: String,
        #[arg(short, long)]
        date: Option<String>,
        #[arg(short, long)]
        tags: Vec<String>,
    },
    List {
        #[arg(value_name = "TYPE")]
        list_type: Option<String>,
        #[arg(short = 'c', long)]
        category: Option<String>,
    },
    Stats {
        #[arg(short = 'c', long)]
        category: Option<String>,
        #[arg(short, long)]
        period: Option<String>,
    },
    Get {
        #[arg(short = 'c', long)]
        category: Option<String>,
        #[arg(short, long)]
        expense_id: Option<String>,
    },
    Update {
        #[arg(value_name = "CATEGORY")]
        category: String,
        #[arg(short, long)]
        amount: Option<f64>,
        #[arg(short, long)]
        period: Option<String>,
        #[arg(short, long)]
        tags: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
    },
    Delete {
        #[arg(short = 'c', long)]
        category: Option<String>,
        #[arg(short, long)]
        expense_id: Option<String>,
    },
    Example {},
    Skill {
        #[arg(value_name = "SUB_COMMAND")]
        sub: Option<String>,
    },
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
            category,
            amount,
            period,
            tags,
            remark,
        } => {
            handle_add(category, amount, period, tags, remark, format)?;
        }
        Commands::Expense {
            category,
            amount,
            description,
            date,
            tags,
        } => {
            handle_expense(category, amount, description, date, tags, format)?;
        }
        Commands::List {
            list_type,
            category,
        } => {
            handle_list(list_type, category, format)?;
        }
        Commands::Stats { category, period } => {
            handle_stats(category, period, format)?;
        }
        Commands::Get {
            category,
            expense_id,
        } => {
            handle_get(category, expense_id, format)?;
        }
        Commands::Update {
            category,
            amount,
            period,
            tags,
            remark,
        } => {
            handle_update(category, amount, period, tags, remark, format)?;
        }
        Commands::Delete {
            category,
            expense_id,
        } => {
            handle_delete(category, expense_id, format)?;
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
    }

    Ok(())
}
