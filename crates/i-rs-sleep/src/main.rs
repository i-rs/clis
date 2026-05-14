use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_stats, handle_update, SkillCommand};
use presentation::OutputFormat;
use crate::presentation::print_error;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-sleep")]
#[command(about = "Sleep tracking CLI - record bedtime, wake time, and sleep quality", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "BEDTIME")]
        bedtime: String,
        #[arg(value_name = "WAKE_TIME")]
        wake_time: String,
        #[arg(value_name = "QUALITY")]
        quality: i32,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "ID")]
        id: String,
    },
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    Get {
        #[arg(value_name = "ID")]
        id: String,
    },
    Stats {},
    Update {
        #[arg(value_name = "ID")]
        id: String,
        #[arg(short, long)]
        bedtime: Option<String>,
        #[arg(short = 'w', long)]
        wake_time: Option<String>,
        #[arg(short, long)]
        quality: Option<i32>,
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
}

fn main() {
    let cli = Cli::parse();
    let format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Table
    };

    if let Err(e) = run(cli.command, format) {
        if cli.json {
            println!("{}", serde_json::json!({
                "success": false,
                "error": { "code": "UNKNOWN", "message": e.to_string() }
            }));
        } else {
            print_error(&format!("{}", e));
        }
        std::process::exit(1);
    }
}

fn run(command: Commands, format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Add { bedtime, wake_time, quality, tag, remark } => {
            handle_add(bedtime, wake_time, quality, tag, remark)?;
        }
        Commands::Delete { id } => {
            handle_delete(id)?;
        }
        Commands::List { tag } => {
            handle_list(tag, format)?;
        }
        Commands::Get { id } => {
            handle_get(id, format)?;
        }
        Commands::Stats {} => {
            handle_stats(format)?;
        }
        Commands::Update { id, bedtime, wake_time, quality, tag, remark } => {
            handle_update(id, bedtime, wake_time, quality, tag, remark)?;
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
