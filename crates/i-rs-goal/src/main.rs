mod commands;
mod models;
mod presentation;
mod storage;

use clap::{Parser, Subcommand};
use commands::{
    add, add_milestone, delete, deposit, example, get, handle_skill, list, list_milestones,
    remove_milestone, stats, update,
};
use presentation::OutputFormat;

#[derive(Parser, Debug)]
#[command(name = "i-rs-goal")]
#[command(about = "Savings goal tracking CLI tool", long_about = None)]
struct Cli {
    #[arg(short, long, help = "Output in JSON format")]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Add a new savings goal")]
    Add(commands::add::AddArgs),

    #[command(about = "List all savings goals")]
    List(commands::list::ListArgs),

    #[command(about = "Get goal details")]
    Get(commands::get::GetArgs),

    #[command(about = "Delete a goal")]
    Delete(commands::delete::DeleteArgs),

    #[command(about = "Update goal properties")]
    Update(commands::update::UpdateArgs),

    #[command(about = "Deposit to a goal")]
    Deposit(commands::deposit::DepositArgs),

    #[command(about = "Manage milestones")]
    Milestone(commands::milestone::MilestoneArgs),

    #[command(about = "View statistics")]
    Stats(commands::stats::StatsArgs),

    #[command(about = "Show usage examples")]
    Example(commands::example::ExampleArgs),

    #[command(about = "Show skill documentation")]
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

fn run(command: Commands, output_format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Add(args) => add(args, output_format),
        Commands::List(args) => list(args, output_format),
        Commands::Get(args) => get(args, output_format),
        Commands::Delete(args) => delete(args, output_format),
        Commands::Update(args) => update(args, output_format),
        Commands::Deposit(args) => deposit(args, output_format),
        Commands::Milestone(args) => {
            if args.list {
                list_milestones(args, output_format)
            } else if args.remove.is_some() {
                remove_milestone(args, output_format)
            } else {
                add_milestone(args, output_format)
            }
        }
        Commands::Stats(args) => stats(args, output_format),
        Commands::Example(args) => example(args),
        Commands::Skill(cmd) => {
            handle_skill(&cmd)?;
            Ok(())
        }
        Commands::Data(commands) => commands::data::handle(&commands),
    }
}

#[cfg(test)]
mod tests;
