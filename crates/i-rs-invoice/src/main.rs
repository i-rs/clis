mod commands;
mod models;
mod presentation;
mod storage;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "i-rs-invoice")]
#[command(about = "Invoice management CLI tool for tracking expense invoices and reimbursement status", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
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
    Skill {
        #[arg(value_name = "SUB_COMMAND")]
        sub: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Add(args) => commands::run_add(args),
        Commands::List(args) => commands::run_list(args),
        Commands::Get(args) => commands::run_get(args),
        Commands::Update(args) => commands::run_update(args),
        Commands::Delete(args) => commands::run_delete(args),
        Commands::Stats(args) => commands::run_stats(args),
        Commands::Example(args) => commands::run_example(args),
        Commands::Skill { sub } => {
            commands::handle_skill(sub.map(|s| match s.as_str() {
                "summary" => commands::SkillCommand::Summary,
                "content" => commands::SkillCommand::Content,
                _ => commands::SkillCommand::Raw,
            }));
            Ok(())
        }
    };

    i_rs_core::exit_on_error!(result, false);
}
