use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_read, handle_skill,
    handle_stats, handle_update,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-article")]
#[command(about = "Article read-later CLI", long_about = None)]
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
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "URL")]
        url: String,
        #[arg(value_name = "TITLE")]
        title: String,
        #[arg(short, long)]
        source: Option<String>,
        #[arg(short = 'g', long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    /// Delete an entry
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// List all entries
    List {
        #[arg(short = 'g', long)]
        tag: Option<String>,
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Update an entry
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short, long)]
        url: Option<String>,
        #[arg(short, long)]
        source: Option<String>,
        #[arg(short, long)]
        status: Option<String>,
        #[arg(short = 'g', long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
        #[arg(short, long)]
        notes: Option<Vec<String>>,
    },
    /// Get an entry by id
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Record a reading entry
    Read {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Show statistics
    Stats {},
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
            name,
            url,
            title,
            source,
            tag,
            remark,
        } => {
            handle_add(
                name,
                title,
                url,
                source.unwrap_or_else(|| "Unknown".to_string()),
                tag,
                remark,
            )?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { tag, status } => {
            handle_list(tag, status, format)?;
        }
        Commands::Update {
            name,
            title,
            url,
            source,
            status,
            tag,
            remark,
            notes,
        } => {
            handle_update(name, title, url, source, status, tag, remark, notes)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
        }
        Commands::Read { name } => {
            handle_read(name)?;
        }
        Commands::Stats {} => {
            handle_stats()?;
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
