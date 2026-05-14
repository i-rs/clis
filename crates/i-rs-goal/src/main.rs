mod commands;
mod models;
mod presentation;
mod storage;

use clap::{Parser, Subcommand};
use commands::*;
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
    Skill {
        #[arg(value_name = "SUB_COMMAND")]
        sub: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let output_format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Table
    };
    
    match cli.command {
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
        Commands::Skill { sub } => {
            commands::handle_skill(sub.map(|s| match s.as_str() {
                "summary" => commands::SkillCommand::Summary,
                "content" => commands::SkillCommand::Content,
                _ => commands::SkillCommand::Raw,
            }));
            Ok(())
        }
    }
}
