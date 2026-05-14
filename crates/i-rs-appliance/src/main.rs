use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_stats, handle_update, SkillCommand};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-appliance")]
#[command(about = "Home appliance lifecycle management CLI", long_about = None)]
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
        #[arg(value_name = "BRAND")]
        brand: String,
        #[arg(value_name = "MODEL")]
        model: String,
        #[arg(value_name = "PURCHASE_DATE")]
        purchase_date: String,
        #[arg(value_name = "LIFESPAN_YEARS")]
        lifespan_years: u32,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    List {
        #[arg(short = 't', long)]
        tag: Option<String>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short = 'b', long)]
        brand: Option<String>,
        #[arg(short = 'm', long)]
        model: Option<String>,
        #[arg(short = 'l', long)]
        lifespan_years: Option<u32>,
        #[arg(short = 't', long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
        #[arg(long)]
        add_maintenance: Option<String>,
        #[arg(long)]
        maintenance_date: Option<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
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
        Commands::Add { name, brand, model, purchase_date, lifespan_years, tag, remark } => {
            handle_add(name, brand, model, purchase_date, lifespan_years, tag, remark)?;
        }
        Commands::List { tag } => {
            handle_list(tag, format)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
        }
        Commands::Update { name, brand, model, lifespan_years, tag, remark, add_maintenance, maintenance_date } => {
            handle_update(name, brand, model, lifespan_years, tag, remark, add_maintenance, maintenance_date)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::Stats {} => {
            handle_stats()?;
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
