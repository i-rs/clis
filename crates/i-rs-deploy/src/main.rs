use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_example, handle_get, handle_list, handle_rollback, handle_skill, handle_stats, SkillCommand};
use models::DeployStatus;
use presentation::OutputFormat;
use crate::presentation::print_error;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-deploy")]
#[command(about = "Deployment record CLI - track deployments, manage rollback, and view statistics", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "PROJECT")]
        project: String,
        #[arg(value_name = "ENVIRONMENT")]
        environment: String,
        #[arg(value_name = "VERSION")]
        version: String,
        #[arg(short, long, default_value = "success")]
        status: DeployStatus,
        #[arg(long)]
        rollback_from: Option<String>,
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
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        env: Option<String>,
        #[arg(short, long)]
        tag: Option<String>,
    },
    Get {
        #[arg(value_name = "ID")]
        id: String,
    },
    Rollback {
        #[arg(value_name = "PROJECT")]
        project: String,
        #[arg(value_name = "ENVIRONMENT")]
        environment: String,
        #[arg(long)]
        rollback_to: Option<String>,
    },
    Stats {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        env: Option<String>,
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
        Commands::Add { project, environment, version, status, rollback_from, tag, remark } => {
            handle_add(project, environment, version, status, rollback_from, tag, remark)?;
        }
        Commands::Delete { id } => {
            handle_delete(id)?;
        }
        Commands::List { project, env, tag } => {
            handle_list(project, env, tag, format)?;
        }
        Commands::Get { id } => {
            handle_get(id, format)?;
        }
        Commands::Rollback { project, environment, rollback_to } => {
            handle_rollback(project, environment, rollback_to)?;
        }
        Commands::Stats { project, env } => {
            handle_stats(project, env)?;
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
