use clap::{Parser, Subcommand};
use commands::{handle_add, handle_checkin, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_update, SkillCommand};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-habit")]
#[command(about = "Habit tracking CLI - build good habits with checkins and streaks", long_about = None)]
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
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long, default_value = "daily")]
        frequency: String,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Checkin {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        frequency: Option<String>,
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
        Commands::Add { name, description, frequency, tag, remark } => {
            handle_add(name, description.unwrap_or_default(), frequency, tag, remark)?;
        }
        Commands::Checkin { name } => {
            handle_checkin(name)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { tag } => {
            handle_list(tag, format)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
        }
        Commands::Update { name, description, frequency, tag, remark } => {
            handle_update(name, description, frequency, tag, remark)?;
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
