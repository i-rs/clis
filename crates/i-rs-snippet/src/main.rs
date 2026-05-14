use clap::{Parser, Subcommand};
use commands::{handle_add, handle_copy, handle_delete, handle_example, handle_get, handle_list, handle_search, handle_skill, handle_update, SkillCommand};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-snippet")]
#[command(about = "Code snippet management CLI", long_about = None)]
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
        language: String,
        #[arg(short, long)]
        code: Vec<String>,
        #[arg(short, long)]
        description: Vec<String>,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    Search {
        #[arg(value_name = "QUERY")]
        query: String,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        language: Option<String>,
        #[arg(short, long)]
        code: Option<Vec<String>>,
        #[arg(short, long)]
        description: Option<Vec<String>>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Copy {
        #[arg(value_name = "NAME")]
        name: String,
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
            name,
            language,
            code,
            description,
            tag,
            remark,
        } => {
            handle_add(name, language, code, description, tag, remark)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { tag } => {
            handle_list(tag, format)?;
        }
        Commands::Search { query } => {
            handle_search(query, format)?;
        }
        Commands::Update {
            name,
            language,
            code,
            description,
            tag,
            remark,
        } => {
            handle_update(name, language, code, description, tag, remark)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
        }
        Commands::Copy { name } => {
            handle_copy(name)?;
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
