#![allow(dead_code)]
use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_example, handle_get, handle_list, handle_rollback, handle_skill, handle_stats, handle_update, parse_skill_arg};
use models::DeployStatus;
use presentation::OutputFormat;

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
    Update {
        #[arg(value_name = "ID")]
        id: String,
        #[arg(short, long, help = "New status")]
        status: Option<DeployStatus>,
        #[arg(short = 'T', long, help = "New tags")]
        tag: Option<Vec<String>>,
        #[arg(short, long, help = "New remarks")]
        remark: Option<Vec<String>>,
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
        Commands::Update { id, status, tag, remark } => {
            handle_update(id, status, tag, remark)?;
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
            handle_skill(parse_skill_arg(sub.as_deref()));
        }
        Commands::Data(commands) => { commands::data::handle(&commands)? }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
