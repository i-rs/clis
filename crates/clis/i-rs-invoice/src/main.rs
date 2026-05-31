mod commands;
mod models;
mod presentation;
mod storage;

use clap::{Parser, Subcommand};
use commands::handle_skill;
use i_rs_core::presentation::OutputFormat;

#[derive(Parser)]
#[command(name = "i-rs-invoice")]
#[command(about = "Invoice management CLI tool for tracking expense invoices and reimbursement status", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Add a new invoice")]
    Add(commands::add::AddArgs),

    #[command(about = "List all invoices")]
    List(commands::list::ListArgs),

    #[command(about = "Get invoice details")]
    Get(commands::get::GetArgs),

    #[command(about = "Update an invoice")]
    Update(commands::update::UpdateArgs),

    #[command(about = "Delete an invoice")]
    Delete(commands::delete::DeleteArgs),

    #[command(about = "View invoice statistics")]
    Stats(commands::stats::StatsArgs),

    #[command(about = "Show usage examples")]
    Example(commands::example::ExampleArgs),

    #[command(about = "Show AI skill documentation")]
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

fn run(command: Commands, _format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Add(args) => commands::run_add(args),
        Commands::List(args) => commands::run_list(args),
        Commands::Get(args) => commands::run_get(args),
        Commands::Update(args) => commands::run_update(args),
        Commands::Delete(args) => commands::run_delete(args),
        Commands::Stats(args) => commands::run_stats(args),
        Commands::Example(args) => commands::run_example(args),
        Commands::Skill(cmd) => {
            handle_skill(&cmd)?;
            Ok(())
        }
        Commands::Data(commands) => commands::data::handle(&commands),
    }
}

#[cfg(test)]
mod tests;
