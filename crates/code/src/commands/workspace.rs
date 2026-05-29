use crate::config;

pub async fn run_workspace_show() -> anyhow::Result<()> {
    let config = config::Config::load()?;
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

pub async fn run_workspace_set(path: &str) -> anyhow::Result<()> {
    let mut config = config::Config::load()?;
    config.workspace = Some(path.to_string());
    config.save()?;
    println!("✓ Workspace set to: {}", path);
    Ok(())
}
