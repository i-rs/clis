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
#[command(name = "i-rs-toothbrush")]
#[command(about = "Toothbrush replacement tracking CLI - record when you replace toothbrushes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new entry
    Add {
        #[arg(value_name = "BRUSH_TYPE")]
        brush_type: String,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    /// Delete an entry
    Delete {
        #[arg(value_name = "ID")]
        id: String,
    },
    /// List all entries
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    /// Get an entry by id
    Get {
        #[arg(value_name = "ID")]
        id: String,
    },
    /// Show usage examples
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
            brush_type,
            tag,
            remark,
        } => {
            handle_add(brush_type, tag, remark)?;
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
