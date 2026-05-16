use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_stats,
    handle_upcoming, handle_update,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-birthday")]
#[command(about = "Birthday reminder CLI", long_about = None)]
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
        #[arg(value_name = "BIRTH_DATE")]
        birth_date: String,
        #[arg(short = 'y', long)]
        year: Option<i32>,
        #[arg(short = 'r', long)]
        relationship: String,
        #[arg(short = 't', long)]
        tag: Vec<String>,
        #[arg(short = 'm', long)]
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
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short = 'd', long)]
        birth_date: Option<String>,
        #[arg(short = 'y', long)]
        year: Option<i32>,
        #[arg(short = 'r', long)]
        relationship: Option<String>,
        #[arg(short = 't', long)]
        tag: Option<Vec<String>>,
        #[arg(short = 'm', long)]
        remark: Option<Vec<String>>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Stats {},
    Upcoming {
        #[arg(short, long)]
        days: Option<i64>,
    },
    Example {},
    #[clap(subcommand)]
    Skill(commands::skill::SkillCommand),
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
        Commands::Add {
            name,
            birth_date,
            year,
            relationship,
            tag,
            remark,
        } => {
            handle_add(name, birth_date, year, relationship, tag, remark)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { tag } => {
            handle_list(tag, format)?;
        }
        Commands::Update {
            name,
            birth_date,
            year,
            relationship,
            tag,
            remark,
        } => {
            handle_update(name, birth_date, year, relationship, tag, remark)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
        }
        Commands::Stats {} => {
            handle_stats(format)?;
        }
        Commands::Upcoming { days } => {
            handle_upcoming(days, format)?;
        }
        Commands::Example {} => {
            handle_example();
        }
        Commands::Skill(cmd) => {
            handle_skill(&cmd)?;
        }
        Commands::Data(commands) => commands::data::handle(&commands)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests;
