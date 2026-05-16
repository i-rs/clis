use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_done, handle_example, handle_get, handle_list, handle_skill,
    handle_update, parse_skill_arg,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod service;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-todo")]
#[command(about = "Todo management CLI", long_about = None)]
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
        #[arg(short = 'p', long)]
        priority: Option<String>,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        content: Vec<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Done {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(short, long)]
        pending: bool,
        #[arg(short, long)]
        done: bool,
        #[arg(short = 't', long)]
        tag: Option<String>,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short = 'p', long)]
        priority: Option<String>,
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
        Commands::Add {
            name,
            title,
            priority,
            tag,
            content,
        } => {
            handle_add(name, title, priority, tag, content)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::Done { name } => {
            handle_done(name)?;
        }
        Commands::List { pending, done, tag } => {
            handle_list(false, pending, done, tag, format)?;
        }
        Commands::Update {
            name,
            title,
            priority,
            tag,
            content,
        } => {
            handle_update(name, title, priority, tag, content)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
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
