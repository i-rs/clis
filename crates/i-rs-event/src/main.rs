#![allow(dead_code)]
mod commands;
mod models;
mod presentation;
mod storage;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "i-rs-event")]
#[command(version = "0.1.0")]
#[command(about = "Social event management CLI tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true, hide = true)]
    pub json: bool,
}

#[derive(Parser, Debug, Clone)]
pub enum Commands {
    #[command(about = "Add a new event")]
    Add(commands::add::AddArgs),

    #[command(about = "List all events")]
    List(commands::list::ListArgs),

    #[command(about = "Get event details")]
    Get(commands::get::GetArgs),

    #[command(about = "Delete an event")]
    Delete(commands::delete::DeleteArgs),

    #[command(about = "Update an event")]
    Update(commands::update::UpdateArgs),

    #[command(about = "View event statistics")]
    Stats(commands::stats::StatsArgs),

    #[command(about = "Show usage examples")]
    Example(commands::example::ExampleArgs),

    #[command(about = "Show AI skill documentation")]
    Skill {
        #[arg(value_name = "SUB_COMMAND")]
        sub: Option<String>,
    },
    #[clap(subcommand)]
    Data(commands::data::DataCommand),
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Add(args) => {
            let json = cli.json;
            commands::add::run(args, json)
        }
        Commands::List(args) => {
            let json = cli.json;
            commands::list::run(args, json)
        }
        Commands::Get(args) => {
            let json = cli.json;
            commands::get::run(args, json)
        }
        Commands::Delete(args) => {
            let json = cli.json;
            commands::delete::run(args, json)
        }
        Commands::Update(args) => {
            let json = cli.json;
            commands::update::run(args, json)
        }
        Commands::Stats(args) => {
            let json = cli.json;
            commands::stats::run(args, json)
        }
        Commands::Example(args) => {
            let json = cli.json;
            commands::example::run(args, json)
        }
        Commands::Skill { sub } => {
            let sub = sub.clone();
            commands::skill::handle_skill(sub.map(|s| match s.as_str() {
                "summary" => commands::skill::SkillCommand::Summary,
                "content" => commands::skill::SkillCommand::Content,
                _ => commands::skill::SkillCommand::Raw,
            }));
            Ok(())
        }
        Commands::Data(commands) => commands::data::handle(commands),
    };

    i_rs_core::exit_on_error!(result, cli.json);
}
