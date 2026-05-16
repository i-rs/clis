use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill,
    handle_update,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod service;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-note")]
#[command(about = "Note management CLI", long_about = None)]
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
        title: Option<String>,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        content: Vec<String>,
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
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        content: Option<Vec<String>>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
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
            title,
            tag,
            content,
        } => {
            handle_add(name, title, tag, content)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { tag } => {
            handle_list(tag, format)?;
        }
        Commands::Update {
            name,
            title,
            tag,
            content,
        } => {
            handle_update(name, title, tag, content)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
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
