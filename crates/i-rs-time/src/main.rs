use clap::{Parser, Subcommand};
use commands::{
    handle_delete, handle_example, handle_get, handle_list, handle_report, handle_skill,
    handle_start, handle_stats, handle_stop, handle_update, parse_skill_arg,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-time")]
#[command(about = "Time tracking CLI for work hours (Pomodoro timer)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Start {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Stop {},
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    Stats {
        #[arg(value_name = "PERIOD")]
        period: String,
    },
    Report {
        #[arg(short, long)]
        start: Option<String>,
        #[arg(short, long)]
        end: Option<String>,
        #[arg(short, long)]
        days: Option<i64>,
    },
    Get {
        #[arg(value_name = "ID")]
        id: String,
    },
    Delete {
        #[arg(value_name = "ID")]
        id: String,
    },
    Update {
        #[arg(value_name = "ID")]
        id: String,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short = 'T', long)]
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
        Commands::Start { name, tag, remark } => {
            handle_start(name, tag, remark, format)?;
        }
        Commands::Stop {} => {
            handle_stop(format)?;
        }
        Commands::List { tag } => {
            handle_list(tag, format)?;
        }
        Commands::Stats { period } => {
            handle_stats(period, format)?;
        }
        Commands::Report { start, end, days } => {
            handle_report(start, end, days, format)?;
        }
        Commands::Get { id } => {
            handle_get(id, format)?;
        }
        Commands::Delete { id } => {
            handle_delete(id, format)?;
        }
        Commands::Update {
            id,
            name,
            tag,
            remark,
        } => {
            handle_update(id, name, tag, remark)?;
        }
        Commands::Example {} => {
            handle_example();
        }
        Commands::Skill { sub } => {
            handle_skill(parse_skill_arg(sub.as_deref()));
        }
        Commands::Data(commands) => commands::data::handle(&commands)?,
    }
    Ok(())
}

#[cfg(test)]
mod tests;
