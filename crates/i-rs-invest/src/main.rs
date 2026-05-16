use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_skill, handle_stats,
    handle_update,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-invest")]
#[command(about = "Investment returns tracking CLI", long_about = None)]
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
        #[arg(short, long = "symbol", value_name = "SYMBOL")]
        symbol: String,
        #[arg(short, long = "type", value_name = "TYPE")]
        asset_type: models::AssetType,
        #[arg(short = 'q', long = "quantity", value_name = "QUANTITY")]
        quantity: f64,
        #[arg(short = 'p', long = "price", value_name = "PRICE")]
        buy_price: f64,
        #[arg(short = 'b', long = "date", value_name = "DATE")]
        buy_date: Option<String>,
        #[arg(long = "current-price", value_name = "PRICE")]
        current_price: Option<f64>,
        #[arg(short = 'g', long = "tag")]
        tag: Vec<String>,
        #[arg(short, long = "remark")]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(short, long = "type", value_name = "TYPE")]
        asset_type: Option<String>,
        #[arg(short = 'g', long = "tag")]
        tag: Option<String>,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long = "symbol", value_name = "SYMBOL")]
        symbol: Option<String>,
        #[arg(short, long = "type", value_name = "TYPE")]
        asset_type: Option<models::AssetType>,
        #[arg(short = 'q', long = "quantity", value_name = "QUANTITY")]
        quantity: Option<f64>,
        #[arg(short = 'p', long = "price", value_name = "PRICE")]
        buy_price: Option<f64>,
        #[arg(long = "current-price", value_name = "PRICE")]
        current_price: Option<f64>,
        #[arg(short = 'g', long = "tag")]
        tag: Option<Vec<String>>,
        #[arg(short, long = "remark")]
        remark: Option<Vec<String>>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Stats {},
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
            symbol,
            asset_type,
            quantity,
            buy_price,
            buy_date,
            current_price,
            tag,
            remark,
        } => {
            handle_add(
                name,
                symbol,
                asset_type,
                quantity,
                buy_price,
                buy_date,
                current_price,
                tag,
                remark,
            )?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::List { asset_type, tag } => {
            handle_list(asset_type, tag, format)?;
        }
        Commands::Update {
            name,
            symbol,
            asset_type,
            quantity,
            buy_price,
            current_price,
            tag,
            remark,
        } => {
            handle_update(
                name,
                symbol,
                asset_type,
                quantity,
                buy_price,
                current_price,
                tag,
                remark,
            )?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
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
