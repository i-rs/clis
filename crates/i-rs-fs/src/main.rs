use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "i-rs-fs")]
#[command(about = "File system operations CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Read {
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    Write {
        #[arg(value_name = "FILE")]
        path: PathBuf,
        #[arg(value_name = "CONTENT")]
        content: String,
    },
    Lines {
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Read { path } => {
            let content = std::fs::read_to_string(&path)?;
            println!("{}", content);
        }
        Commands::Write { path, content } => {
            std::fs::write(&path, content)?;
            println!("Written to {}", path.display());
        }
        Commands::Lines { path } => {
            let content = std::fs::read_to_string(&path)?;
            let lines = content.lines().count();
            println!("{} lines in {}", lines, path.display());
        }
    }

    Ok(())
}
