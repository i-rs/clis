mod commands;
mod models;
mod presentation;
mod storage;

use clap::{Parser, Subcommand};
use i_rs_core::presentation::OutputFormat;
use commands::{handle_skill, parse_skill_arg};

#[derive(Parser, Debug)]
#[command(name = "i-rs-plant")]
#[command(about = "Plant care tracking CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

fn get_output_format(cli: &Cli) -> OutputFormat {
    if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Table
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Add a new plant")]
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        location: String,
        #[arg(long, default_value = "7")]
        interval: u32,
        #[arg(long, short, action = clap::ArgAction::Append)]
        tag: Vec<String>,
        #[arg(long, short, action = clap::ArgAction::Append)]
        remark: Vec<String>,
    },
    #[command(about = "List all plants")]
    List {
        #[arg(long, short)]
        tag: Option<String>,
    },
    #[command(about = "Get plant details")]
    Get {
        #[command(subcommand)]
        subcommand: Option<GetSubcommands>,
    },
    #[command(about = "Water a plant")]
    Water {
        #[command(subcommand)]
        subcommand: Option<WaterSubcommands>,
    },
    #[command(about = "Update plant info")]
    Update {
        #[command(subcommand)]
        subcommand: Option<UpdateSubcommands>,
    },
    #[command(about = "Delete a plant")]
    Delete {
        #[command(subcommand)]
        subcommand: Option<DeleteSubcommands>,
    },
    #[command(about = "View statistics")]
    Stats,
    #[command(about = "Show usage examples")]
    Example,
    #[command(about = "Show skill documentation")]
    Skill {
        #[arg(value_name = "SUB_COMMAND")]
        sub: Option<String>,
    },
#[clap(subcommand)]
Data(commands::data::DataCommand),
}

#[derive(Subcommand, Debug)]
enum GetSubcommands {
    #[command(about = "Get plant details")]
    Name { name: String },
}

#[derive(Subcommand, Debug)]
enum WaterSubcommands {
    #[command(about = "Water a plant")]
    Name { name: String },
}

#[derive(Subcommand, Debug)]
enum UpdateSubcommands {
    #[command(about = "Update plant info")]
    Name {
        name: String,
        #[arg(long)]
        species: Option<String>,
        #[arg(long)]
        location: Option<String>,
        #[arg(long)]
        interval: Option<u32>,
        #[arg(long, short, action = clap::ArgAction::Append)]
        tag: Vec<String>,
        #[arg(long, short, action = clap::ArgAction::Append)]
        remark: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
enum DeleteSubcommands {
    #[command(about = "Delete a plant")]
    Name { name: String },
}

fn main() {
    let cli = Cli::parse();
    let format = get_output_format(&cli);
    i_rs_core::exit_on_error!(run(cli.command, format), cli.json);
}

fn run(command: Commands, output_format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Add {
            name,
            species,
            location,
            interval,
            tag,
            remark,
        } => {
            commands::add_plant(name, species, location, interval, tag, remark, output_format)?;
        }
        Commands::List { tag } => {
            commands::list_plants(tag, output_format)?;
        }
        Commands::Get { subcommand } => {
            if let Some(GetSubcommands::Name { name }) = subcommand {
                commands::get_plant(name, output_format)?;
            } else {
                anyhow::bail!("Usage: i-rs-plant get <name>");
            }
        }
        Commands::Water { subcommand } => {
            if let Some(WaterSubcommands::Name { name }) = subcommand {
                commands::water_plant(name, output_format)?;
            } else {
                anyhow::bail!("Usage: i-rs-plant water <name>");
            }
        }
        Commands::Update { subcommand } => {
            if let Some(UpdateSubcommands::Name {
                name,
                species,
                location,
                interval,
                tag,
                remark,
            }) = subcommand
            {
                commands::update_plant(
                    name,
                    species,
                    location,
                    interval,
                    if tag.is_empty() { None } else { Some(tag) },
                    if remark.is_empty() { None } else { Some(remark) },
                    output_format,
                )?;
            } else {
                anyhow::bail!("Usage: i-rs-plant update <name> [options]");
            }
        }
        Commands::Delete { subcommand } => {
            if let Some(DeleteSubcommands::Name { name }) = subcommand {
                commands::delete_plant(name, output_format)?;
            } else {
                anyhow::bail!("Usage: i-rs-plant delete <name>");
            }
        }
        Commands::Stats => {
            commands::stats(output_format)?;
        }
        Commands::Example => {
            commands::example();
        }
        Commands::Skill { sub } => {
            handle_skill(parse_skill_arg(sub.as_deref()));
        },
        Commands::Data(commands) => { commands::data::handle(&commands)? }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
