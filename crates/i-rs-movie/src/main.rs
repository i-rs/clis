use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill,
    handle_stats, handle_update, handle_watch, SkillCommand,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-movie")]
#[command(about = "Movie tracking CLI", long_about = None)]
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
        year: Option<i32>,
        #[arg(short, long)]
        director: Option<String>,
        #[arg(short, long)]
        watched: bool,
        #[arg(short, long)]
        rating: Option<f32>,
        #[arg(long)]
        review: Vec<String>,
        #[arg(long)]
        release_date: Option<String>,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(long)]
        watched: bool,
        #[arg(long)]
        unwatched: bool,
        #[arg(short, long)]
        tag: Option<String>,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        year: Option<i32>,
        #[arg(short, long)]
        director: Option<String>,
        #[arg(short, long)]
        rating: Option<f32>,
        #[arg(long)]
        review: Option<Vec<String>>,
        #[arg(long)]
        release_date: Option<String>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(long)]
        remark: Option<Vec<String>>,
    },
    Watch {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        rating: Option<f32>,
        #[arg(long)]
        review: Option<Vec<String>>,
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

    if let Err(e) = run(cli.command, format) {
        if cli.json {
            println!("{}", serde_json::json!({
                "success": false,
                "error": { "code": "UNKNOWN", "message": e.to_string() }
            }));
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }
}

fn run(command: Commands, format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Add {
            name,
            year,
            director,
            watched,
            rating,
            review,
            release_date,
            tag,
            remark,
        } => {
            handle_add(name, year, director, watched, rating, review, release_date, tag, remark, format)?;
        }
        Commands::Delete { name } => {
            handle_delete(name, format)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
        }
        Commands::List {
            watched,
            unwatched,
            tag,
        } => {
            let watch_filter = if watched {
                Some(true)
            } else if unwatched {
                Some(false)
            } else {
                None
            };
            handle_list(watch_filter, tag, format)?;
        }
        Commands::Update {
            name,
            year,
            director,
            rating,
            review,
            release_date,
            tag,
            remark,
        } => {
            handle_update(name, year, director, rating, review, release_date, tag, remark, format)?;
        }
        Commands::Watch { name, rating, review } => {
            handle_watch(name, rating, review, format)?;
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
