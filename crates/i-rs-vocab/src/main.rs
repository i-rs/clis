use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_quiz, handle_skill,
    handle_stats, handle_update, parse_skill_arg,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-vocab")]
#[command(about = "Vocabulary learning CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "WORD")]
        word: String,
        #[arg(value_name = "DEFINITION")]
        definition: String,
        #[arg(short = 'e', long)]
        example: Vec<String>,
        #[arg(short = 's', long)]
        status: Option<String>,
        #[arg(short = 't', long)]
        tag: Vec<String>,
        #[arg(short = 'r', long)]
        remark: Vec<String>,
    },
    Get {
        #[arg(value_name = "WORD")]
        word: String,
    },
    List {
        #[arg(short = 's', long)]
        status: Option<String>,
        #[arg(short = 't', long)]
        tag: Option<String>,
    },
    Update {
        #[arg(value_name = "WORD")]
        word: String,
        #[arg(short = 'd', long)]
        definition: Option<String>,
        #[arg(short = 'e', long)]
        example: Option<Vec<String>>,
        #[arg(short = 's', long)]
        status: Option<String>,
        #[arg(short = 't', long)]
        tag: Option<Vec<String>>,
        #[arg(short = 'r', long)]
        remark: Option<Vec<String>>,
        #[arg(long)]
        review: bool,
    },
    Delete {
        #[arg(value_name = "WORD")]
        word: String,
    },
    Quiz {
        #[arg(short = 'c', long)]
        count: Option<usize>,
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
        Commands::Add {
            word,
            definition,
            example,
            status,
            tag,
            remark,
        } => {
            handle_add(word, definition, example, status, tag, remark)?;
        }
        Commands::Get { word } => {
            handle_get(word, format)?;
        }
        Commands::List { status, tag } => {
            handle_list(status, tag, format)?;
        }
        Commands::Update {
            word,
            definition,
            example,
            status,
            tag,
            remark,
            review,
        } => {
            handle_update(word, definition, example, status, tag, remark, review)?;
        }
        Commands::Delete { word } => {
            handle_delete(word)?;
        }
        Commands::Quiz { count } => {
            handle_quiz(count)?;
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
        Commands::Data(commands) => commands::data::handle(&commands)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests;
