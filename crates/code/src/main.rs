mod cli;
mod config;
mod app;
mod agent;
mod tools;
mod provider;
mod protocol;
mod diff;
mod session;
mod utils;
mod tui;

use clap::Parser;
use cli::Cli;
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        cli::Commands::Tui => {
            #[cfg(feature = "tui")]
            {
                let app = app::App::new(config);
                tui::run(app).await?;
            }
            #[cfg(not(feature = "tui"))]
            {
                anyhow::bail!("TUI feature not enabled. Build with --features tui");
            }
        }
        cli::Commands::Chat { prompt, json } => {
            let provider = provider::create_provider(&config)?;
            let tools = tools::ToolRegistry::new(&config)?;
            let mut agent = agent::Agent::new(config, provider, tools, json);
            agent.run_once(&prompt).await?;
        }
        cli::Commands::Agent { task_id } => {
            let provider = provider::create_provider(&config)?;
            let tools = tools::ToolRegistry::new(&config)?;
            let mut agent = agent::Agent::new(config, provider, tools, true);
            protocol::handler::run_agent_loop(&mut agent, &task_id).await?;
        }
        cli::Commands::Config { .. } => {
            println!("Config file: {:?}", config::config_path());
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
    }
    Ok(())
}
