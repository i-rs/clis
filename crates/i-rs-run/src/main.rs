#![allow(dead_code)]
use clap::{Parser, Subcommand};
use crate::presentation::print_error;
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_plan_add,
    handle_plan_delete, handle_plan_get, handle_plan_list, handle_skill, handle_stats,
    handle_update, SkillCommand,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-run")]
#[command(about = "Running record CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "DATE")]
        date: String,
        #[arg(value_name = "DISTANCE")]
        distance: f64,
        #[arg(value_name = "DURATION")]
        duration: f64,
        #[arg(long)]
        heart_rate: Option<u32>,
        #[arg(short = 'w', long)]
        weather: Option<String>,
        #[arg(short = 't', long)]
        tags: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    List {},
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
        date: Option<String>,
        #[arg(short = 'd', long)]
        distance: Option<f64>,
        #[arg(short = 'u', long)]
        duration: Option<f64>,
        #[arg(short = 'r', long)]
        heart_rate: Option<Option<u32>>,
        #[arg(short = 'w', long)]
        weather: Option<Option<String>>,
        #[arg(short = 'T', long)]
        tags: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
    },
    Stats {},
    PlanAdd {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "TARGET")]
        target: f64,
        #[arg(value_name = "PACE")]
        pace: String,
        #[arg(short = 's', long, num_args = 1..)]
        schedule: Vec<u8>,
        #[arg(short = 't', long)]
        tags: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    PlanList {},
    PlanGet {
        #[arg(value_name = "ID")]
        id: String,
    },
    PlanDelete {
        #[arg(value_name = "ID")]
        id: String,
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
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": { "code": "UNKNOWN", "message": e.to_string() }
                })
            );
        } else {
            print_error(&format!("{}", e));
        }
        std::process::exit(1);
    }
}

fn run(command: Commands, format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Add {
            date,
            distance,
            duration,
            heart_rate,
            weather,
            tags,
            remark,
        } => {
            handle_add(date, distance, duration, heart_rate, weather, tags, remark)?;
        }
        Commands::List {} => {
            handle_list(format)?;
        }
        Commands::Get { id } => {
            handle_get(id, format)?;
        }
        Commands::Delete { id } => {
            handle_delete(id)?;
        }
        Commands::Update { id, date, distance, duration, heart_rate, weather, tags, remark } => {
            handle_update(id, date, distance, duration, heart_rate, weather, tags, remark)?;
        }
        Commands::Stats {} => {
            handle_stats()?;
        }
        Commands::PlanAdd {
            name,
            target,
            pace,
            schedule,
            tags,
            remark,
        } => {
            handle_plan_add(name, target, pace, schedule, tags, remark)?;
        }
        Commands::PlanList {} => {
            handle_plan_list(format)?;
        }
        Commands::PlanGet { id } => {
            handle_plan_get(id, format)?;
        }
        Commands::PlanDelete { id } => {
            handle_plan_delete(id)?;
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
