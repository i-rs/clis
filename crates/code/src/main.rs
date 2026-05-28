// Some modules are only exercised through specific feature paths or CLI subcommands.
#![allow(dead_code)]

pub mod error;
mod cli;
mod config;
mod app;
mod agent;
mod tools;
mod provider;
mod protocol;
mod memory;
mod convstore;
mod router;
mod diff;
mod session;
mod utils;
mod debug;
#[cfg(test)]
mod testing;
mod lsp;
mod pty;
mod mcp;
mod prompt;
mod runtime;
mod tui;

use clap::Parser;
use cli::{Cli, ConfigCommands};
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    if cli.debug {
        crate::runtime::set_debug(true);
    }
    if cli.verbose {
        crate::runtime::set_verbose(true);
    }

    match cli.command {
        cli::Commands::Tui { session } => {
            #[cfg(feature = "tui")]
            {
                if config.api_key.as_ref().is_none_or(|k| k.trim().is_empty()) {
                    println!("⚠  API key not configured. The AI agent won't work until you set it up.");
                    println!("   Run:  i-rs-code config init");
                    println!();
                }
                let mut app = app::App::new(config.clone(), session.clone());
                let tools = tools::ToolRegistry::new(&config)?;
                app.tool_names = tools.schemas().iter()
                    .filter_map(|s| s.get("function").and_then(|f| f.get("name")).and_then(|n| n.as_str()).map(String::from))
                    .collect();
                if let Some(ref sid) = session {
                    let sessions_dir = config::i_rs_code_dir().join("sessions");
                    if let Ok(s) = session::Session::load(sid, &sessions_dir) {
                        app.messages = s.messages;
                    }
                }
                tui::run(app).await?;
            }
            #[cfg(not(feature = "tui"))]
            {
                anyhow::bail!("TUI feature not enabled. Build with --features tui");
            }
        }
        cli::Commands::Chat { prompt, json } => {
            if config.api_key.as_ref().is_none_or(|k| k.trim().is_empty()) {
                anyhow::bail!("API key not configured. Run `i-rs-code config init` to set up.");
            }
            let provider = provider::create_provider(&config)?;
            let tools = tools::ToolRegistry::new(&config)?;
            let mut agent = agent::Agent::new(config, provider, tools, json);
            agent.run_once(&prompt).await?;
        }
        cli::Commands::Agent { task_id } => {
            if config.api_key.as_ref().is_none_or(|k| k.trim().is_empty()) {
                anyhow::bail!("API key not configured. Run `i-rs-code config init` to set up.");
            }
            let provider = provider::create_provider(&config)?;
            let tools = tools::ToolRegistry::new(&config)?;
            let mut agent = agent::Agent::new(config, provider, tools, true);
            protocol::handler::run_agent_loop(&mut agent, &task_id).await?;
        }
        cli::Commands::Config(cmd) => match cmd {
            ConfigCommands::Show => {
                println!("Config file: {:?}", config::config_path());
                println!("{}", serde_json::to_string_pretty(&config)?);
            }
            ConfigCommands::Init => {
                run_config_init().await?;
            }
            ConfigCommands::Set { key, value } => {
                run_config_set(config, &key, &value).await?;
            }
        },
        cli::Commands::Search { query, limit } => {
            let sessions_dir = config::i_rs_code_dir().join("sessions");
            let store = convstore::ConvStore::new(sessions_dir);
            let results = store.search(&query, limit);
            if results.is_empty() {
                println!("No results found for: {}", query);
            } else {
                println!("Found {} results:", results.len());
                for (i, r) in results.iter().enumerate() {
                    println!("  {}. [{}] {} ({})", i + 1, r.message_type, r.excerpt, r.session_id);
                }
            }
        },
    }
    Ok(())
}

async fn run_config_init() -> anyhow::Result<()> {
    use std::io::{self, Write};

    let path = config::config_path();
    println!("Config file: {:?}", path);

    let mut config = Config::load()?;

    println!();
    println!("── i-rs-code 配置向导 ──");
    println!();

    fn prompt(label: &str, default: &str, buf: &mut String) -> io::Result<()> {
        print!("{} [{}]: ", label, default);
        io::stdout().flush()?;
        buf.clear();
        io::stdin().read_line(buf)?;
        Ok(())
    }

    let mut input = String::new();

    // Provider
    prompt("LLM provider", &config.provider, &mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        config.provider = trimmed;
    }

    // API key
    let masked = config.api_key.as_ref().map(|k| {
        if k.len() > 8 {
            format!("{}...{}", &k[..4], &k[k.len()-4..])
        } else {
            "****".to_string()
        }
    });
    prompt("API key", &masked.unwrap_or_else(|| "not set".into()), &mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        config.api_key = Some(trimmed);
    }

    // Base URL
    prompt("Base URL", config.effective_base_url(), &mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        config.base_url = Some(trimmed);
    }

    // Model
    prompt("Model", config.effective_model(), &mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        config.model = Some(trimmed);
    }

    config.save()?;
    println!();
    println!("✓ Configuration saved to {:?}", path);
    println!("  Provider: {}", config.provider);
    println!("  Model:    {}", config.effective_model());
    println!("  Base URL: {}", config.effective_base_url());

    Ok(())
}

async fn run_config_set(mut config: Config, key: &str, value: &str) -> anyhow::Result<()> {
    match key {
        "provider" => config.provider = value.to_string(),
        "api_key" => config.api_key = Some(value.to_string()),
        "base_url" => config.base_url = Some(value.to_string()),
        "model" => config.model = Some(value.to_string()),
        "workspace" => config.workspace = Some(value.to_string()),
        _ => anyhow::bail!("Unknown config key: {}. Valid keys: provider, api_key, base_url, model, workspace", key),
    }
    config.save()?;
    println!("✓ {} set to {}", key, value);
    Ok(())
}
