use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_copy, handle_delete, handle_example, handle_get, handle_list,
    handle_rename, handle_search, handle_skill, handle_stats, handle_update,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod service;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-kv")]
#[command(about = "Key-value storage CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "KEY")]
        key: String,
        #[arg(value_name = "VALUE")]
        value: String,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "KEY")]
        key: String,
    },
    List {
        #[arg(short, long)]
        tag: Option<String>,
        #[arg(short, long)]
        pattern: Option<String>,
        #[arg(short = 'L', long)]
        limit: Option<usize>,
        #[arg(short = 'O', long)]
        offset: Option<usize>,
    },
    Update {
        #[arg(value_name = "KEY")]
        key: String,
        #[arg(short, long)]
        value: Option<String>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
    },
    Get {
        #[arg(value_name = "KEY")]
        key: String,
    },
    Search {
        #[arg(value_name = "QUERY")]
        query: String,
    },
    Stats {},
    Copy {
        #[arg(value_name = "SRC")]
        src: String,
        #[arg(value_name = "DST")]
        dst: String,
    },
    Rename {
        #[arg(value_name = "OLD_KEY")]
        old: String,
        #[arg(value_name = "NEW_KEY")]
        new: String,
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
        Commands::Add { key, value, tag, remark } => {
            handle_add(key, value, tag, remark, format)?;
        }
        Commands::Delete { key } => {
            handle_delete(key, format)?;
        }
        Commands::List {
            tag,
            pattern,
            limit,
            offset,
        } => {
            handle_list(tag, pattern, limit, offset, format)?;
        }
        Commands::Update {
            key,
            value,
            tag,
            remark,
        } => {
            handle_update(key, value, tag, remark, format)?;
        }
        Commands::Get { key } => {
            handle_get(key, format)?;
        }
        Commands::Search { query } => {
            handle_search(query, format)?;
        }
        Commands::Stats {} => {
            handle_stats(format)?;
        }
        Commands::Copy { src, dst } => {
            handle_copy(src, dst, format)?;
        }
        Commands::Rename { old, new } => {
            handle_rename(old, new, format)?;
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
