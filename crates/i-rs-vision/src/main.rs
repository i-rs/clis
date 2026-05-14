use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_stats, SkillCommand};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-vision")]
#[command(about = "Vision tracking CLI", long_about = None)]
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
        #[arg(long = "left-sphere", value_name = "D")]
        left_sphere: Option<f64>,
        #[arg(long = "right-sphere", value_name = "D")]
        right_sphere: Option<f64>,
        #[arg(long = "left-cylinder", value_name = "D")]
        left_cylinder: Option<f64>,
        #[arg(long = "right-cylinder", value_name = "D")]
        right_cylinder: Option<f64>,
        #[arg(long = "left-axis", value_name = "DEG")]
        left_axis: Option<i32>,
        #[arg(long = "right-axis", value_name = "DEG")]
        right_axis: Option<i32>,
        #[arg(short = 't', long)]
        tag: Vec<String>,
        #[arg(short = 'r', long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "DATE")]
        date: String,
    },
    List {
        #[arg(short = 'd', long)]
        days: Option<usize>,
    },
    Get {
        #[arg(value_name = "DATE")]
        date: String,
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
            date,
            left_sphere,
            right_sphere,
            left_cylinder,
            right_cylinder,
            left_axis,
            right_axis,
            tag,
            remark,
        } => {
            handle_add(
                date,
                left_sphere,
                right_sphere,
                left_cylinder,
                right_cylinder,
                left_axis,
                right_axis,
                tag,
                remark,
            )?;
        }
        Commands::Delete { date } => {
            handle_delete(date)?;
        }
        Commands::List { days } => {
            handle_list(days, format)?;
        }
        Commands::Get { date } => {
            handle_get(date, format)?;
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
