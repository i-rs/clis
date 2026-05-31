mod agent;
mod app;
mod cli;
mod commands;
mod config;
mod convstore;
mod debug;
mod diff;
pub mod error;
mod lsp;
mod mcp;
mod memory;
mod prompt;
mod protocol;
mod provider;
mod pty;
mod router;
mod runtime;
mod session;
mod skill_store;
#[cfg(test)]
mod testing;
mod tokenizer;
mod tools;
mod tui;
mod utils;

use clap::Parser;
use cli::{
    Cli, Commands, ConfigCommands, McpCommands, PluginsCommands, SessionsCommands, SkillCommands,
    SystemPromptCommands,
};
use config::Config;

fn init_tracing() {
    let filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .without_time()
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let config = Config::load()?;

    if cli.debug {
        crate::runtime::set_debug(true);
    }
    if cli.verbose {
        crate::runtime::set_verbose(true);
    }

    match &cli.command {
        Commands::Tui { session } => {
            #[cfg(feature = "tui")]
            {
                if config.api_key.as_ref().is_none_or(|k| k.trim().is_empty()) {
                    println!(
                        "⚠  API key not configured. The AI agent won't work until you set it up."
                    );
                    println!("   Run:  i-rs-code config init");
                    println!();
                }
                let mut app = app::App::new(config.clone(), session.clone());
                let tools = tools::ToolRegistry::new(&config)?;
                app.tool_names = tools
                    .schemas()
                    .iter()
                    .filter_map(|s| {
                        s.get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(|n| n.as_str())
                            .map(String::from)
                    })
                    .collect();
                if let Some(sid) = session {
                    let sessions_dir = config::i_rs_code_dir().join("sessions");
                    if let Ok(s) = session::Session::load(sid, &sessions_dir) {
                        app.messages = s.messages;
                        app.agent_messages = s.agent_messages;
                    }
                }
                tui::run(app).await?;
            }
            #[cfg(not(feature = "tui"))]
            {
                anyhow::bail!("TUI feature not enabled. Build with --features tui");
            }
        }
        Commands::Chat { prompt, json } => {
            if config.api_key.as_ref().is_none_or(|k| k.trim().is_empty()) {
                anyhow::bail!("API key not configured. Run `i-rs-code config init` to set up.");
            }
            let provider = provider::create_provider(&config)?;
            let tools = tools::ToolRegistry::new(&config)?;
            let mut agent = agent::Agent::new(config, provider, tools, *json);
            agent.run_once(prompt).await?;
        }
        Commands::Agent { task_id } => {
            if config.api_key.as_ref().is_none_or(|k| k.trim().is_empty()) {
                anyhow::bail!("API key not configured. Run `i-rs-code config init` to set up.");
            }
            let provider = provider::create_provider(&config)?;
            let tools = tools::ToolRegistry::new(&config)?;
            let mut agent = agent::Agent::new(config, provider, tools, true);
            protocol::handler::run_agent_loop(&mut agent, task_id).await?;
        }
        Commands::Config(cmd) => match cmd {
            ConfigCommands::Show => {
                println!("Config file: {:?}", config::config_path());
                println!("{}", serde_json::to_string_pretty(&config)?);
            }
            ConfigCommands::Init => {
                commands::run_config_init().await?;
            }
            ConfigCommands::Set { key, value } => {
                commands::run_config_set(key, value).await?;
            }
        },
        Commands::SystemPrompt(cmd) => match cmd {
            SystemPromptCommands::Show { full } => {
                let (content, source) = if *full {
                    let project_info = config::ProjectInfo::detect();
                    (
                        prompt::build_system_prompt(&project_info),
                        "built dynamically".to_string(),
                    )
                } else {
                    (prompt::load_system_prompt(), {
                        let path = prompt::system_prompt_path();
                        if path.exists() {
                            format!("file: {:?}", path)
                        } else {
                            "built-in default".to_string()
                        }
                    })
                };
                println!("── System Prompt ({}) ──", source);
                println!();
                println!("{}", content);
            }
            SystemPromptCommands::Reset => {
                let path = prompt::write_default_prompt_file()?;
                println!("✓ System prompt reset to default at {:?}", path);
            }
            SystemPromptCommands::Dir => {
                let dir = prompt::prompt_dir();
                println!("{}", dir.display());
            }
        },
        Commands::Search { query, limit } => {
            let sessions_dir = config::i_rs_code_dir().join("sessions");
            let store = convstore::ConvStore::new(sessions_dir);
            let results = store.search(query, *limit);
            if results.is_empty() {
                println!("No results found for: {}", query);
            } else {
                println!("Found {} results:", results.len());
                for (i, r) in results.iter().enumerate() {
                    println!(
                        "  {}. [{}] {} ({})",
                        i + 1,
                        r.message_type,
                        r.excerpt,
                        r.session_id
                    );
                }
            }
        }
        Commands::Version => {
            commands::show_version(&config);
        }
        Commands::Sessions(cmd) => match cmd {
            SessionsCommands::List => {
                commands::run_sessions_list().await?;
            }
            SessionsCommands::Show { id, full } => {
                commands::run_sessions_show(id, *full).await?;
            }
            SessionsCommands::Delete { id } => {
                commands::run_sessions_delete(id).await?;
            }
            SessionsCommands::Export { id } => {
                commands::run_sessions_export(id).await?;
            }
        },
        Commands::Workspace { path } => {
            if let Some(p) = path {
                commands::run_workspace_set(p).await?;
            } else {
                commands::run_workspace_show().await?;
            }
        }
        Commands::Doctor => {
            commands::run_doctor().await?;
        }
        Commands::Mcp(cmd) => match cmd {
            McpCommands::List => {
                commands::run_mcp_list();
            }
            McpCommands::Add {
                name,
                command,
                args,
                url,
                env,
            } => {
                commands::run_mcp_add(name, command, args, url, env).await?;
            }
            McpCommands::Remove { name } => {
                commands::run_mcp_remove(name).await?;
            }
            McpCommands::Test { name } => {
                commands::run_mcp_test(name).await?;
            }
        },
        Commands::Plugins(cmd) => match cmd {
            PluginsCommands::List => {
                commands::run_plugins_list().await?;
            }
            PluginsCommands::Dir => {
                commands::run_plugins_dir().await?;
            }
        },
        Commands::Skill(cmd) => match cmd {
            SkillCommands::List => {
                commands::run_skill_list();
            }
            SkillCommands::Get { name } => {
                commands::run_skill_get(name);
            }
            SkillCommands::Create { name, description } => {
                commands::run_skill_create(name, description)?;
            }
        },
    }
    Ok(())
}
