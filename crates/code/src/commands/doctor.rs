use crate::config::{self, Config};

pub async fn run_doctor() -> anyhow::Result<()> {
    let config = Config::load()?;
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
