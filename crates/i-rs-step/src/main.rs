use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_update, SkillCommand};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-step")]
#[command(about = "Step tracking CLI - record daily step counts", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "STEPS")]
        steps: i32,
        #[arg(value_name = "DATE")]
        date: String,
        #[arg(short, long)]
        distance: Option<f64>,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "DATE")]
        date: String,
    },
    List {},
    Get {
        #[arg(value_name = "DATE")]
        date: String,
    },
    Update {
        #[arg(value_name = "DATE")]
        date: String,
        #[arg(short, long)]
        steps: Option<i32>,
        #[arg(short, long)]
        distance: Option<f64>,
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

    i_rs_core::exit_on_error!(run(cli.command, format), cli.json);
}

fn run(command: Commands, format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Add { steps, date, distance, tag, remark } => {
            handle_add(steps, date, distance, tag, remark)?;
        }
        Commands::Delete { date } => {
            handle_delete(date)?;
        }
        Commands::List {} => {
            handle_list(format)?;
        }
        Commands::Get { date } => {
            handle_get(date, format)?;
        }
        Commands::Update { date, steps, distance, tag, remark } => {
            handle_update(date, steps, distance, tag, remark)?;
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
