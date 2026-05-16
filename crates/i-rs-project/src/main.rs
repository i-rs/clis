use clap::{Parser, Subcommand};
use commands::{
    handle_add, handle_delete, handle_example, handle_get, handle_list, handle_milestone,
    handle_skill, handle_stats, handle_task, handle_update, milestone::MilestoneCommand,
    task::TaskCommand,
};
use presentation::OutputFormat;

mod commands;
mod models;
mod presentation;
mod storage;

#[derive(Parser, Debug)]
#[command(name = "i-rs-project")]
#[command(about = "Project management CLI", long_about = None)]
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
        #[arg(short = 'd', long)]
        description: Option<String>,
        #[arg(short = 's', long)]
        status: Option<String>,
        #[arg(short = 'p', long)]
        priority: Option<String>,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(short = 't', long)]
        tag: Option<String>,
        #[arg(short = 's', long)]
        status: Option<String>,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short = 'd', long)]
        description: Option<String>,
        #[arg(short = 's', long)]
        status: Option<String>,
        #[arg(short = 'p', long)]
        priority: Option<String>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        remark: Option<Vec<String>>,
    },
    Stats {},
    Milestone {
        #[command(subcommand)]
        command: MilestoneCommand,
    },
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    Example {},
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

fn run(command: Commands, format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Add {
            name,
            description,
            status,
            priority,
            tag,
            remark,
        } => {
            handle_add(name, description, status, priority, tag, remark)?;
        }
        Commands::Delete { name } => {
            handle_delete(name)?;
        }
        Commands::Get { name } => {
            handle_get(name, format)?;
        }
        Commands::List { tag, status } => {
            handle_list(tag, status, format)?;
        }
        Commands::Update {
            name,
            description,
            status,
            priority,
            tag,
            remark,
        } => {
            handle_update(name, description, status, priority, tag, remark)?;
        }
        Commands::Stats {} => {
            handle_stats()?;
        }
        Commands::Milestone { command } => {
            handle_milestone(command, format)?;
        }
        Commands::Task { command } => {
            handle_task(command)?;
        }
        Commands::Example {} => {
            handle_example();
        }
        Commands::Skill(cmd) => {
            handle_skill(&cmd)?;
        }
        Commands::Data(commands) => commands::data::handle(&commands)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests;
