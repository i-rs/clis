use anyhow::Result;
use clap::{Parser, Subcommand};
use i_rs_core::presentation::OutputFormat;
use crate::presentation::print_error;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-read")]
#[command(about = "Reading progress tracking CLI tool", long_about = None)]
struct Cli {
    #[arg(short, long, help = "Output in JSON format", global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Add a new book")]
    Add(commands::add::AddArgs),

    #[command(about = "Delete a book")]
    Delete(commands::delete::DeleteArgs),

    #[command(about = "Get book details")]
    Get(commands::get::GetArgs),

    #[command(about = "List all books")]
    List(commands::list::ListArgs),

    #[command(about = "Update book information")]
    Update(commands::update::UpdateArgs),

    #[command(about = "Show reading statistics")]
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
    if let Err(e) = run() {
        print_error(&format!("{}", e));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let output_format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Table
    };

    match cli.command {
        Commands::Add(args) => commands::add(args, output_format)?,
        Commands::Delete(args) => commands::delete(args, output_format)?,
        Commands::Get(args) => commands::get(args, output_format)?,
        Commands::List(args) => commands::list(args, output_format)?,
        Commands::Update(args) => commands::update(args, output_format)?,
        Commands::Stats(args) => commands::stats(args, output_format)?,
        Commands::Example(args) => commands::example(args)?,
        Commands::Skill { sub } =>
            commands::handle_skill(sub.map(|s| match s.as_str() {
                "summary" => commands::SkillCommand::Summary,
                "content" => commands::SkillCommand::Content,
                _ => commands::SkillCommand::Raw,
            })),
    }

    Ok(())
}
