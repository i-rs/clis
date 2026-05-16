use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-bed")]
#[command(about = "Bed item replacement tracking CLI - record when you replace mattress, pillows", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "ITEM_TYPE")]
        item_type: String,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "ID")]
        id: String,
    },
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    Get {
        #[arg(value_name = "ID")]
        id: String,
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
            item_type,
            tag,
            remark,
        } => {
            handle_add(item_type, tag, remark)?;
        }
        Commands::Delete { id } => {
            handle_delete(id)?;
        }
        Commands::List { tag } => {
            handle_list(tag, format)?;
        }
        Commands::Get { id } => {
            handle_get(id, format)?;
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
