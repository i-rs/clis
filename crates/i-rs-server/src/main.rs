use clap::{Parser, Subcommand};
use commands::{handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_suggest, handle_update, parse_skill_arg};
use presentation::OutputFormat;
use storage::init_keyring;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-server")]
#[command(about = "Server management CLI", long_about = None)]
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
        #[arg(value_name = "HOST")]
        host: String,
        #[arg(value_name = "PORT")]
        port: Option<u16>,
        #[arg(short, long)]
        user: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
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
        #[arg(short, long)]
        tag: Option<String>,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(long)]
        host: Option<String>,
        #[arg(short = 'P', long)]
        port: Option<u16>,
        #[arg(short, long)]
        user: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short = 's', long)]
        show_password: bool,
    },
    Suggest {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        command: Option<String>,
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
    init_keyring();

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
            host,
            port,
            user,
            password,
            tag,
            remark,
        } => {
            handle_add(name, host, port, user, password, tag, remark)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { tag } => {
            handle_list(tag, format)?;
        }
        Commands::Update {
            name,
            host,
            port,
            user,
            password,
            tag,
            remark,
        } => {
            handle_update(name, host, port, user, password, tag, remark)?;
        }
        Commands::Get {
            name,
            show_password,
        } => {
            handle_get(name, show_password, format)?;
        }
        Commands::Suggest { name, command } => {
            handle_suggest(name, command)?;
        }
        Commands::Example {} => {
            handle_example();
        }
        Commands::Skill { sub } => {
            handle_skill(parse_skill_arg(sub.as_deref()));
        }
        Commands::Data(commands) => { commands::data::handle(&commands)? }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
