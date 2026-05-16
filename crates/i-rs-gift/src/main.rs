use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_stats,
    handle_update, parse_skill_arg,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-gift")]
#[command(about = "Gift management CLI", long_about = None)]
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
        #[arg(value_name = "TYPE")]
        gift_type: String,
        #[arg(value_name = "RECIPIENT")]
        recipient: String,
        #[arg(value_name = "OCCASION")]
        occasion: String,
        #[arg(value_name = "VALUE")]
        value: f64,
        #[arg(value_name = "DATE")]
        date: String,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(long)]
        gift_type: Option<String>,
        #[arg(short, long)]
        tag: Option<String>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short = 'y', long)]
        gift_type: Option<String>,
        #[arg(short = 'r', long)]
        recipient: Option<String>,
        #[arg(short = 'o', long)]
        occasion: Option<String>,
        #[arg(short = 'v', long)]
        value: Option<f64>,
        #[arg(short = 'd', long)]
        date: Option<String>,
        #[arg(short = 'T', long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
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
            name,
            gift_type,
            recipient,
            occasion,
            value,
            date,
            tag,
            remark,
        } => {
            handle_add(
                name, gift_type, recipient, occasion, value, date, tag, remark,
            )?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { gift_type, tag } => {
            handle_list(tag, gift_type, format)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
        }
        Commands::Update {
            name,
            gift_type,
            recipient,
            occasion,
            value,
            date,
            tag,
            remark,
        } => {
            handle_update(
                name, gift_type, recipient, occasion, value, date, tag, remark,
            )?;
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
