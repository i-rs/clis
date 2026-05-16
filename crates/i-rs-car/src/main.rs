use clap::{Parser, Subcommand};
use commands::{add, delete, example, fuel, get, list, maintain, stats, update};
use commands::skill::{handle_skill, parse_skill_arg};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-car")]
#[command(about = "Vehicle management CLI", long_about = None)]
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
        #[arg(short = 'l', long = "license-plate")]
        license_plate: String,
        #[arg(short = 'b', long)]
        brand: String,
        #[arg(short = 'm', long)]
        model: String,
        #[arg(short = 'i', long)]
        mileage: f64,
        #[arg(short = 't', long)]
        tags: Option<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    List {
        #[arg(short, long)]
        car: Option<String>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        fuel: bool,
        #[arg(short, long)]
        maintain: bool,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        rename: Option<String>,
        #[arg(short = 'l', long = "license-plate")]
        license_plate: Option<String>,
        #[arg(short = 'b', long)]
        brand: Option<String>,
        #[arg(short = 'm', long)]
        model: Option<String>,
        #[arg(short = 'i', long)]
        mileage: Option<f64>,
        #[arg(short, long)]
        add_tags: Option<String>,
        #[arg(short, long)]
        remove_tags: Option<String>,
        #[arg(short, long)]
        add_remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        force: bool,
    },
    Fuel {
        #[arg(help = "Car name")]
        #[arg(value_name = "CAR")]
        car: String,
        #[arg(short = 'd', long)]
        date: Option<String>,
        #[arg(short = 'i', long)]
        mileage: f64,
        #[arg(short = 'f', long)]
        fuel_amount: f64,
        #[arg(short = 'p', long)]
        price: f64,
        #[arg(short, long)]
        fuel_type: Option<String>,
        #[arg(short = 's', long)]
        station: Option<String>,
        #[arg(short = 'n', long)]
        note: Option<String>,
    },
    Maintain {
        #[arg(help = "Car name")]
        #[arg(value_name = "CAR")]
        car: String,
        #[arg(short = 'd', long)]
        date: Option<String>,
        #[arg(short = 'i', long)]
        mileage: f64,
        #[arg(short = 't', long)]
        maintenance_type: String,
        #[arg(short = 'c', long)]
        cost: f64,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        shop: Option<String>,
        #[arg(short = 'n', long)]
        note: Option<String>,
    },
    Stats {
        #[arg(short, long)]
        car: Option<String>,
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
        Commands::Add { name, license_plate, brand, model, mileage, tags, remark } => {
            add::run(&add::Args { name, license_plate, brand, model, mileage, tags, remark }, format)?;
        }
        Commands::List { car } => {
            list::run(&list::Args { car }, format)?;
        }
        Commands::Get { name, fuel, maintain } => {
            get::run(&get::Args { name, fuel, maintain }, format)?;
        }
        Commands::Update { name, rename, license_plate, brand, model, mileage, add_tags, remove_tags, add_remark } => {
            update::run(&update::Args { name, rename, license_plate, brand, model, mileage, add_tags, remove_tags, add_remark }, format)?;
        }
        Commands::Delete { name, force } => {
            delete::run(&delete::Args { name, force })?;
        }
        Commands::Fuel { car, date, mileage, fuel_amount, price, fuel_type, station, note } => {
            fuel::run(&fuel::Args { car, date, mileage, fuel_amount, price, fuel_type, station, note }, format)?;
        }
        Commands::Maintain { car, date, mileage, maintenance_type, cost, description, shop, note } => {
            maintain::run(&maintain::Args { car, date, mileage, maintenance_type, cost, description, shop, note }, format)?;
        }
        Commands::Stats { car } => {
            stats::run(&stats::Args { car }, format)?;
        }
        Commands::Example {} => {
            example::run(&example::Args {})?;
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
