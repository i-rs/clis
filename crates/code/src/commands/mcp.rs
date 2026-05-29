use crate::config::{self, Config};

pub fn run_mcp_list() {
    let config = Config::load().unwrap_or_default();
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

pub async fn run_mcp_add(
    name: &str,
    command: &str,
    args: &Option<Vec<String>>,
    url: &Option<String>,
    env: &Option<Vec<String>>,
) -> anyhow::Result<()> {
    let mut config = Config::load()?;
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

    config.mcp_servers.push(server);
    config.save()?;
    println!("✓ MCP server '{}' added.", name);
    Ok(())
}

pub async fn run_mcp_remove(name: &str) -> anyhow::Result<()> {
    let mut config = Config::load()?;
    let idx = config.mcp_servers.iter().position(|s| s.name == name)
        .ok_or_else(|| anyhow::anyhow!("MCP server '{}' not found.", name))?;

    config.mcp_servers.remove(idx);
    config.save()?;
    println!("✓ MCP server '{}' removed.", name);
    Ok(())
}

pub async fn run_mcp_test(name: &str) -> anyhow::Result<()> {
    let config = Config::load()?;
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

    println!();
    println!("Connection closed.");
    Ok(())
}
