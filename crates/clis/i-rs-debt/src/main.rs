mod commands;
mod models;
mod presentation;
mod storage;

use anyhow::Result;
use clap::Parser;
use commands::handle_skill;
use presentation::OutputFormat;

#[derive(Parser, Debug)]
#[command(
    name = "i-rs-debt",
    about = "Debt management CLI tool",
    long_about = None
)]
struct Cli {
    #[arg(short, long, help = "Output in JSON format")]
    json: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    #[command(about = "Add a new debt")]
    Add(commands::add::Args),
    #[command(about = "List all debts")]
    List(commands::list::Args),
    #[command(about = "Get debt details")]
    Get(commands::get::Args),
    #[command(about = "Delete a debt")]
    Delete(commands::delete::Args),
    #[command(about = "Update debt information")]
    Update(commands::update::Args),
    #[command(about = "Record a payment")]
    Pay(commands::pay::Args),
    #[command(about = "Show debt statistics")]
    Stats(commands::stats::Args),
    #[command(about = "Show usage examples")]
    Example(commands::example::Args),
    #[command(about = "Show AI skill documentation")]
    #[clap(subcommand)]
    Skill(commands::skill::SkillCommand),
    #[clap(subcommand)]
    Data(commands::data::DataCommand),
}

fn main() {
    let cli = Cli::parse();

    let output_format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Table
    };

    i_rs_core::exit_on_error!(run(cli.command, output_format), cli.json);
}

fn run(command: Commands, output_format: OutputFormat) -> Result<()> {
    match command {
        Commands::Add(args) => commands::add::run(&args, output_format),
        Commands::List(args) => commands::list::run(&args, output_format),
        Commands::Get(args) => commands::get::run(&args, output_format),
        Commands::Delete(args) => commands::delete::run(&args, output_format),
        Commands::Update(args) => commands::update::run(&args, output_format),
        Commands::Pay(args) => commands::pay::run(&args, output_format),
        Commands::Stats(args) => commands::stats::run(&args, output_format),
        Commands::Example(args) => commands::example::run(&args),
        Commands::Skill(cmd) => {
            handle_skill(&cmd)?;
            Ok(())
        }
        Commands::Data(commands) => commands::data::handle(&commands),
    }
}

#[cfg(test)]
mod tests;
