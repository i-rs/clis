use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_stats, handle_update, SkillCommand};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-exercise")]
#[command(about = "Exercise tracking CLI - track your workouts and fitness activities", long_about = None)]
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
        #[arg(value_name = "TYPE")]
        exercise_type: String,
        #[arg(value_name = "DURATION")]
        duration_minutes: u32,
        #[arg(short = 'c', long)]
        calories: Option<u32>,
        #[arg(short = 't', long)]
        tag: Vec<String>,
        #[arg(short = 'n', long)]
        notes: Vec<String>,
        #[arg(short = 'r', long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(short = 't', long)]
        tag: Option<String>,
        #[arg(short = 'y', long)]
        exercise_type: Option<String>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short = 'y', long)]
        exercise_type: Option<String>,
        #[arg(short = 'd', long)]
        duration_minutes: Option<u32>,
        #[arg(short = 'c', long)]
        calories: Option<Option<u32>>,
        #[arg(short = 'n', long)]
        notes: Option<Vec<String>>,
        #[arg(short = 't', long)]
        tag: Option<Vec<String>>,
        #[arg(short = 'r', long)]
        remark: Option<Vec<String>>,
    },
    Stats {},
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
        Commands::Add { name, exercise_type, duration_minutes, calories, tag, notes, remark } => {
            handle_add(name, exercise_type, duration_minutes, calories, notes, tag, remark)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { tag, exercise_type } => {
            handle_list(tag, exercise_type, format)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
        }
        Commands::Update { name, exercise_type, duration_minutes, calories, notes, tag, remark } => {
            handle_update(name, exercise_type, duration_minutes, calories, notes, tag, remark)?;
        }
        Commands::Stats {} => {
            handle_stats(format)?;
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
