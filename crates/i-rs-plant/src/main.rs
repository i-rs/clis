mod commands;
mod models;
mod presentation;
mod storage;

use anyhow::Result;
use clap::{Parser, Subcommand};
use i_rs_core::presentation::OutputFormat;

#[derive(Parser, Debug)]
#[command(name = "i-rs-plant")]
#[command(about = "Plant care tracking CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, default_value = "table")]
    format: String,
}

fn get_output_format(format_str: &str) -> OutputFormat {
    match format_str.to_lowercase().as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Table,
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
        #[arg(default_value = "content")]
        args: Vec<String>,
    },
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let output_format = get_output_format(&cli.format);

    match cli.command {
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
                println!("Usage: i-rs-plant get <name>");
                std::process::exit(1);
            }
        }
        Commands::Water { subcommand } => {
            if let Some(WaterSubcommands::Name { name }) = subcommand {
                commands::water_plant(name, output_format)?;
            } else {
                println!("Usage: i-rs-plant water <name>");
                std::process::exit(1);
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
                println!("Usage: i-rs-plant update <name> [options]");
                std::process::exit(1);
            }
        }
        Commands::Delete { subcommand } => {
            if let Some(DeleteSubcommands::Name { name }) = subcommand {
                commands::delete_plant(name, output_format)?;
            } else {
                println!("Usage: i-rs-plant delete <name>");
                std::process::exit(1);
            }
        }
        Commands::Stats => {
            commands::stats(output_format)?;
        }
        Commands::Example => {
            commands::example();
        }
        Commands::Skill { args } => {
            commands::skill(args);
        }
    }

    Ok(())
}
