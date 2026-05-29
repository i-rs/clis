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
mod skill_store;
mod runtime;
mod tokenizer;
mod tui;

use clap::Parser;
use cli::{Cli, Commands, ConfigCommands, SessionsCommands, McpCommands, PluginsCommands, SkillCommands};
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

    match &cli.command {
        Commands::Tui { session } => {
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
                run_config_init().await?;
            }
            ConfigCommands::Set { key, value } => {
                let config = config.clone();
                run_config_set(config, key, value).await?;
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
                    println!("  {}. [{}] {} ({})", i + 1, r.message_type, r.excerpt, r.session_id);
                }
            }
        }
        Commands::Version => {
            show_version(&config);
        }
        Commands::Sessions(cmd) => match cmd {
            SessionsCommands::List => {
                run_sessions_list(&config).await?;
            }
            SessionsCommands::Show { id, full } => {
                run_sessions_show(&config, id, *full).await?;
            }
            SessionsCommands::Delete { id } => {
                run_sessions_delete(&config, id).await?;
            }
            SessionsCommands::Export { id } => {
                run_sessions_export(&config, id).await?;
            }
        },
        Commands::Workspace { path } => {
            if let Some(p) = path {
                run_workspace_set(&config, p).await?;
            } else {
                run_workspace_show(&config).await?;
            }
        }
        Commands::Doctor => {
            run_doctor(&config).await?;
        }
        Commands::Mcp(cmd) => match cmd {
            McpCommands::List => {
                run_mcp_list(&config);
            }
            McpCommands::Add { name, command, args, url, env } => {
                run_mcp_add(&config, name, command, args, url, env).await?;
            }
            McpCommands::Remove { name } => {
                run_mcp_remove(&config, name).await?;
            }
            McpCommands::Test { name } => {
                run_mcp_test(&config, name).await?;
            }
        },
        Commands::Plugins(cmd) => match cmd {
            PluginsCommands::List => {
                run_plugins_list(&config).await?;
            }
            PluginsCommands::Dir => {
                run_plugins_dir(&config).await?;
            }
        },
        Commands::Skill(cmd) => match cmd {
            SkillCommands::List => {
                run_skill_list();
            }
            SkillCommands::Get { name } => {
                run_skill_get(name);
            }
            SkillCommands::Create { name, description } => {
                run_skill_create(name, description)?;
            }
        },
    }
    Ok(())
}

// ── Version ──

fn show_version(config: &Config) {
    println!("i-rs-code {}", env!("CARGO_PKG_VERSION"));
    println!("Provider:  {}", config.provider);
    println!("Model:     {}", config.effective_model());
    println!("Config:    {}", config::config_path().display());
    println!("Data dir:  {}", config::i_rs_code_dir().display());
    if let Some(ref ws) = config.workspace {
        println!("Workspace: {}", ws);
    }
}

// ── Sessions ──

async fn run_sessions_list(_config: &Config) -> anyhow::Result<()> {
    let sessions_dir = config::i_rs_code_dir().join("sessions");
    if !sessions_dir.exists() {
        println!("No sessions found.");
        return Ok(());
    }

    let mut entries: Vec<(String, String, usize, String)> = Vec::new();
    for entry in std::fs::read_dir(&sessions_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let session: session::Session = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => continue,
        };
        entries.push((
            session.id,
            session.created_at,
            session.messages.len(),
            session.updated_at,
        ));
    }

    entries.sort_by(|a, b| b.1.cmp(&a.1));

    if entries.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }

    println!("Sessions ({} total):", entries.len());
    println!();
    for (id, created, count, updated) in &entries {
        let short_id = if id.len() > 8 { &id[..8] } else { id.as_str() };
        println!("  {:<12} │ {} msgs │ created: {} │ updated: {}", short_id, count, created, updated);
    }
    println!();
    println!("Use `i-rs-code sessions show <id>` to view details.");
    Ok(())
}

async fn run_sessions_show(_config: &Config, id: &str, full: bool) -> anyhow::Result<()> {
    let sessions_dir = config::i_rs_code_dir().join("sessions");
    let session = session::Session::load(id, &sessions_dir)?;

    println!("Session: {}", session.id);
    println!("Created: {}", session.created_at);
    println!("Updated: {}", session.updated_at);
    println!("Messages: {}", session.messages.len());
    println!();

    for (i, msg) in session.messages.iter().enumerate() {
        match msg {
            app::AgentMessage::User { content } => {
                println!("── [{}. User] ──", i + 1);
                print_content(content, full);
            }
            app::AgentMessage::Assistant { content, reasoning, .. } => {
                println!("── [{}. Assistant] ──", i + 1);
                if !reasoning.is_empty() {
                    println!("  [reasoning]: {}", truncate(reasoning, 200));
                }
                print_content(content, full);
            }
            app::AgentMessage::ToolResult { content } => {
                println!("── [{}. Tool Result] ──", i + 1);
                let preview: String = content.chars().take(300).collect();
                if content.len() > 300 {
                    println!("  {}...", preview);
                } else {
                    println!("  {}", preview);
                }
            }
            app::AgentMessage::System { content } => {
                println!("── [{}. System] ──", i + 1);
                print_content(content, full);
            }
            app::AgentMessage::FileEdit { path, summary } => {
                println!("── [{}. File Edit] ──", i + 1);
                println!("  {}: {}", path, summary);
            }
            app::AgentMessage::Separator { label } => {
                println!("  ── {} ──", label);
            }
        }
    }
    Ok(())
}

async fn run_sessions_delete(_config: &Config, id: &str) -> anyhow::Result<()> {
    let sessions_dir = config::i_rs_code_dir().join("sessions");
    let path = sessions_dir.join(format!("{}.json", id));
    if !path.exists() {
        anyhow::bail!("Session '{}' not found at {:?}", id, path);
    }
    std::fs::remove_file(&path)?;
    println!("✓ Session '{}' deleted.", id);
    Ok(())
}

async fn run_sessions_export(_config: &Config, id: &str) -> anyhow::Result<()> {
    let sessions_dir = config::i_rs_code_dir().join("sessions");
    let session = session::Session::load(id, &sessions_dir)?;

    println!("# Session: {}", session.id);
    println!();
    println!("- **Created**: {}", session.created_at);
    println!("- **Updated**: {}", session.updated_at);
    println!("- **Messages**: {}", session.messages.len());
    println!();

    for msg in &session.messages {
        match msg {
            app::AgentMessage::User { content } => {
                println!("## User\n");
                println!("{}", content);
                println!();
            }
            app::AgentMessage::Assistant { content, reasoning, .. } => {
                println!("## Assistant\n");
                if !reasoning.is_empty() {
                    println!("> **Reasoning**: {}\n", reasoning);
                }
                println!("{}", content);
                println!();
            }
            app::AgentMessage::ToolResult { content } => {
                println!("### Tool Result\n");
                println!("```");
                println!("{}", content);
                println!("```");
                println!();
            }
            app::AgentMessage::System { content } => {
                println!("### System\n");
                println!("```");
                println!("{}", content);
                println!("```");
                println!();
            }
            app::AgentMessage::FileEdit { path, summary } => {
                println!("### File Edit: `{}`", path);
                println!("{}", summary);
                println!();
            }
            app::AgentMessage::Separator { label } => {
                println!("--- *{}* ---", label);
                println!();
            }
        }
    }
    Ok(())
}

// ── Workspace ──

async fn run_workspace_show(config: &Config) -> anyhow::Result<()> {
    match &config.workspace {
        Some(ws) => {
            let exists = std::path::Path::new(ws).exists();
            println!("Workspace: {}", ws);
            if !exists {
                println!("  ⚠  Directory does not exist.");
            }
        }
        None => {
            println!("Workspace: (not set)");
            println!("  Current directory: {:?}", std::env::current_dir().unwrap_or_default());
            println!("  Set with: i-rs-code workspace <path>");
        }
    }
    Ok(())
}

async fn run_workspace_set(config: &Config, path: &str) -> anyhow::Result<()> {
    let mut config = config.clone();
    config.workspace = Some(path.to_string());
    config.save()?;
    println!("✓ Workspace set to: {}", path);
    Ok(())
}

// ── Doctor ──

async fn run_doctor(config: &Config) -> anyhow::Result<()> {
    println!("── i-rs-code diagnostic ──");
    println!();

    // 1. Config file
    let config_path = config::config_path();
    println!("[1/6] Config file");
    if config_path.exists() {
        println!("  ✓ {:?}", config_path);
    } else {
        println!("  ✗ Not found at {:?}", config_path);
    }

    // 2. API key
    println!();
    println!("[2/6] API key");
    let env_key = std::env::var("I_RS_CODE_API_KEY").ok();
    let effective_key = config.api_key.as_ref().or(env_key.as_ref());
    match effective_key {
        Some(k) if !k.trim().is_empty() => {
            let masked = if k.len() > 8 {
                format!("{}...{}", &k[..4], &k[k.len() - 4..])
            } else {
                "****".to_string()
            };
            let source = if std::env::var("I_RS_CODE_API_KEY").is_ok() {
                "env var I_RS_CODE_API_KEY"
            } else {
                "config file"
            };
            println!("  ✓ Key found ({} from {})", masked, source);
        }
        _ => {
            println!("  ✗ Not configured");
            println!("     Run: i-rs-code config init");
        }
    }

    // 3. Provider
    println!();
    println!("[3/6] Provider");
    match config.provider.as_str() {
        "openai" | "anthropic" | "ollama" => {
            println!("  ✓ {}", config.provider);
            println!("     Model: {}", config.effective_model());
            println!("     Base URL: {}", config.effective_base_url());
        }
        other => {
            println!("  ✗ Unknown provider: '{}'", other);
            println!("     Supported: openai, anthropic, ollama");
        }
    }

    // 4. Workspace
    println!();
    println!("[4/6] Workspace");
    match &config.workspace {
        Some(ws) => {
            let path = std::path::Path::new(ws);
            if path.exists() {
                println!("  ✓ {}", ws);
            } else {
                println!("  ⚠  Configured but directory not found: {}", ws);
            }
        }
        None => {
            println!("  - Not set (current dir: {})", std::env::current_dir().unwrap_or_default().display());
        }
    }

    // 5. MCP servers
    println!();
    println!("[5/6] MCP servers");
    if config.mcp_servers.is_empty() {
        println!("  - None configured");
    } else {
        for server in &config.mcp_servers {
            let transport = &server.transport_type;
            let target = server.command.as_deref().or(server.url.as_deref()).unwrap_or("?");
            println!("  ✓ {} ({}: {})", server.name, transport, target);
        }
    }

    // 6. Plugin dirs
    println!();
    println!("[6/6] Plugin directories");
    let tools_dir = config.tools_dir();
    let bin_dir = config.bin_dir();
    println!("  Tools: {}", tools_dir.display());
    println!("  Bin:   {}", bin_dir.display());
    if tools_dir.exists() {
        let count = std::fs::read_dir(&tools_dir)
            .map(|e| e.flatten().count())
            .unwrap_or(0);
        println!("         ({} entries)", count);
    } else {
        println!("         (does not exist)");
    }
    if bin_dir.exists() {
        let count = std::fs::read_dir(&bin_dir)
            .map(|e| e.flatten().count())
            .unwrap_or(0);
        println!("         ({} entries)", count);
    } else {
        println!("         (does not exist)");
    }

    println!();
    println!("── Done ──");
    Ok(())
}

// ── MCP ──

fn run_mcp_list(config: &Config) {
    if config.mcp_servers.is_empty() {
        println!("No MCP servers configured.");
        println!("Add one with: i-rs-code mcp add <name> --command <cmd>");
        return;
    }

    println!("Configured MCP servers:");
    for server in &config.mcp_servers {
        let transport = &server.transport_type;
        let target = server.command.as_deref().or(server.url.as_deref()).unwrap_or("?");
        println!("  {} ({}: {})", server.name, transport, target);
    }
}

async fn run_mcp_add(
    config: &Config,
    name: &str,
    command: &str,
    args: &Option<Vec<String>>,
    url: &Option<String>,
    env: &Option<Vec<String>>,
) -> anyhow::Result<()> {
    if config.mcp_servers.iter().any(|s| s.name == name) {
        anyhow::bail!("MCP server '{}' already configured.", name);
    }

    let transport_type = if url.is_some() { "sse".into() } else { "stdio".into() };
    let server = config::McpServerConfig {
        name: name.to_string(),
        transport_type,
        command: Some(command.to_string()),
        args: args.clone(),
        url: url.clone(),
        env: env.clone(),
    };

    let mut config = config.clone();
    config.mcp_servers.push(server);
    config.save()?;
    println!("✓ MCP server '{}' added.", name);
    Ok(())
}

async fn run_mcp_remove(config: &Config, name: &str) -> anyhow::Result<()> {
    let idx = config.mcp_servers.iter().position(|s| s.name == name)
        .ok_or_else(|| anyhow::anyhow!("MCP server '{}' not found.", name))?;

    let mut config = config.clone();
    config.mcp_servers.remove(idx);
    config.save()?;
    println!("✓ MCP server '{}' removed.", name);
    Ok(())
}

async fn run_mcp_test(config: &Config, name: &str) -> anyhow::Result<()> {
    let server = config.mcp_servers.iter().find(|s| s.name == name)
        .ok_or_else(|| anyhow::anyhow!("MCP server '{}' not found.", name))?;

    if server.transport_type != "stdio" {
        anyhow::bail!("Only stdio transport is supported for testing (got: {})", server.transport_type);
    }

    let command = server.command.as_deref()
        .ok_or_else(|| anyhow::anyhow!("No command configured for MCP server '{}'", name))?;
    let args: Vec<&str> = server.args.as_ref()
        .map(|a| a.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();

    println!("Connecting to MCP server '{}'...", name);
    println!("  Command: {} {}", command, args.join(" "));
    println!();

    let mgr = crate::mcp::McpManager::new();
    let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    mgr.connect(name, command, &args_owned).await?;
    let tools = mgr.discover_tools(name).await?;

    if tools.is_empty() {
        println!("✓ Connected, but no tools discovered.");
    } else {
        println!("✓ Connected. Discovered {} tools:", tools.len());
        for tool in &tools {
            let desc = if tool.description.len() > 80 {
                format!("{}...", &tool.description[..77])
            } else {
                tool.description.clone()
            };
            println!("  • {}: {}", tool.name, desc);
        }
    }

    // Connection drops here, child process killed
    println!();
    println!("Connection closed.");
    Ok(())
}

// ── Plugins ──

async fn run_plugins_list(config: &Config) -> anyhow::Result<()> {
    let tools_dir = config.tools_dir();
    let bin_dir = config.bin_dir();

    println!("Plugin directories:");

    println!("\n  Tools dir: {}", tools_dir.display());
    if tools_dir.exists() {
        let mut has_entries = false;
        for entry in std::fs::read_dir(&tools_dir)?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let meta = entry.metadata().ok();
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let kind = if meta.as_ref().map_or(false, |m| m.is_dir()) { "dir" } else { "file" };
            println!("    {:30} {}  {} bytes", name, kind, size);
            has_entries = true;
        }
        if !has_entries {
            println!("    (empty)");
        }
    } else {
        println!("    (does not exist)");
    }

    println!("\n  Bin dir: {}", bin_dir.display());
    if bin_dir.exists() {
        let mut has_entries = false;
        for entry in std::fs::read_dir(&bin_dir)?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let meta = entry.metadata().ok();
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let kind = if meta.as_ref().map_or(false, |m| m.is_dir()) { "dir" } else { "file" };
            println!("    {:30} {}  {} bytes", name, kind, size);
            has_entries = true;
        }
        if !has_entries {
            println!("    (empty)");
        }
    } else {
        println!("    (does not exist)");
    }

    Ok(())
}

async fn run_plugins_dir(config: &Config) -> anyhow::Result<()> {
    let tools_dir = config.tools_dir();
    let bin_dir = config.bin_dir();
    println!("Tools dir: {}", tools_dir.display());
    println!("Bin dir:   {}", bin_dir.display());
    println!();
    println!("Config keys: tools_dir / bin_dir in config.toml");
    println!("Env var:     I_RS_CODE_DIR");
    Ok(())
}

// ── Skills ──

fn run_skill_list() {
    let store = skill_store::SkillStore::new();
    let skills = store.list();
    if skills.is_empty() {
        println!("No skills installed.");
        println!();
        println!("Skills directory: {:?}", store.dir());
        println!("Create one with:  i-rs-code skill create <name> --description \"...\"");
        return;
    }
    println!("Skills directory: {:?}", store.dir());
    println!();
    println!("Installed skills ({}):", skills.len());
    for skill in &skills {
        println!("  {}  — {}", skill.name, skill.description);
    }
}

fn run_skill_get(name: &str) {
    let store = skill_store::SkillStore::new();
    match store.get(name) {
        Some(skill) => {
            println!("── Skill: {} ──", skill.name);
            println!();
            println!("{}", skill.content);
        }
        None => {
            println!("Skill '{}' not found.", name);
            println!("Use `i-rs-code skill list` to see available skills.");
        }
    }
}

fn run_skill_create(name: &str, description: &str) -> anyhow::Result<()> {
    match skill_store::create_skill(name, description) {
        Ok(path) => {
            println!("✓ Skill '{}' created at {:?}", name, path);
            println!("  Edit this file to add your skill instructions.");
            Ok(())
        }
        Err(e) => {
            Err(e)
        }
    }
}

// ── Config Init & Set (existing) ──

async fn run_config_init() -> anyhow::Result<()> {
    use std::io::{self, Write};

    let path = config::config_path();
    println!("Config file: {:?}", path);

    let mut config = Config::load()?;

    println!();
    println!("── i-rs-code configuration wizard ──");
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
        config.api_key = Some(trimmed.clone());
        if !trimmed.starts_with("$") {
            eprintln!("\n  Warning: API key stored in plaintext. Consider using I_RS_CODE_API_KEY env var instead.");
        }
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

    let gitignore_path = crate::config::i_rs_code_dir().join(".gitignore");
    if !gitignore_path.exists() {
        let _ = std::fs::write(&gitignore_path, "*\n");
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

// ── Helpers ──

fn print_content(content: &str, full: bool) {
    if full {
        println!("{}", content);
    } else if content.len() > 500 {
        println!("{}...", &content[..497]);
        println!("  (use --full to see all {} characters)", content.len());
    } else {
        println!("{}", content);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
