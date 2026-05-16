#![allow(dead_code)]
use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_stats, handle_update, parse_skill_arg};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-cycling")]
#[command(about = "Cycling record tracking CLI", long_about = None)]
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
        duration: u32,
        #[arg(short = 'e', long)]
        elevation: Option<f64>,
        #[arg(short = 'r', long)]
        route: Option<String>,
        #[arg(short = 't', long)]
        tag: Vec<String>,
        #[arg(short = 'm', long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "ID|DATE")]
        id_or_date: String,
    },
    Get {
        #[arg(value_name = "ID|DATE")]
        id_or_date: String,
    },
    List {
        #[arg(short = 't', long)]
        tag: Option<String>,
    },
    Update {
        #[arg(value_name = "ID|DATE")]
        id_or_date: String,
        #[arg(short = 'd', long)]
        distance: Option<f64>,
        #[arg(short = 'u', long)]
        duration: Option<u32>,
        #[arg(short = 'e', long)]
        elevation: Option<Option<f64>>,
        #[arg(short = 'r', long)]
        route: Option<String>,
        #[arg(long)]
        add_tag: Option<String>,
        #[arg(long)]
        remove_tag: Option<String>,
        #[arg(long)]
        add_remark: Option<String>,
    },
    Stats {},
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
        Commands::Add { date, distance, duration, elevation, route, tag, remark } => {
            handle_add(date, distance, duration, elevation, route, tag, remark)?;
        }
        Commands::Delete { id_or_date } => {
            handle_delete(id_or_date)?;
        }
        Commands::Get { id_or_date } => {
            handle_get(id_or_date, format)?;
        }
        Commands::List { tag } => {
            handle_list(tag, format)?;
        }
        Commands::Update { id_or_date, distance, duration, elevation, route, add_tag, remove_tag, add_remark } => {
            handle_update(id_or_date, distance, duration, elevation, route, add_tag, remove_tag, add_remark)?;
        }
        Commands::Stats {} => {
            handle_stats()?;
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
