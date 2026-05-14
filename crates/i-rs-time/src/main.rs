#![allow(dead_code)]
use clap::{Parser, Subcommand};
use commands::{handle_start, handle_stop, handle_list, handle_stats, handle_report, handle_delete, handle_get, handle_update, handle_example, handle_skill, SkillCommand};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-time")]
#[command(about = "Time tracking CLI for work hours (Pomodoro timer)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Start {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Stop {},
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    Stats {
        #[arg(value_name = "PERIOD")]
        period: String,
    },
    Report {
        #[arg(short, long)]
        start: Option<String>,
        #[arg(short, long)]
        end: Option<String>,
        #[arg(short, long)]
        days: Option<i64>,
    },
    Get {
        #[arg(value_name = "ID")]
        id: String,
    },
    Delete {
        #[arg(value_name = "ID")]
        id: String,
    },
    Update {
        #[arg(value_name = "ID")]
        id: String,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short = 'T', long)]
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

    i_rs_core::exit_on_error!(run(cli.command, format), cli.json);
}

fn run(command: Commands, format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Start { name, tag, remark } => {
            handle_start(name, tag, remark, format)?;
        }
        Commands::Stop {} => {
            handle_stop(format)?;
        }
        Commands::List { tag } => {
            handle_list(tag, format)?;
        }
        Commands::Stats { period } => {
            handle_stats(period, format)?;
        }
        Commands::Report { start, end, days } => {
            handle_report(start, end, days, format)?;
        }
        Commands::Get { id } => {
            handle_get(id, format)?;
        }
        Commands::Delete { id } => {
            handle_delete(id, format)?;
        }
        Commands::Update { id, name, tag, remark } => {
            handle_update(id, name, tag, remark)?;
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
