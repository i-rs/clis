use crate::config::Config;

pub async fn run_plugins_list() -> anyhow::Result<()> {
    let config = Config::load()?;
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

pub async fn run_plugins_dir() -> anyhow::Result<()> {
    let config = Config::load()?;
    let tools_dir = config.tools_dir();
    let bin_dir = config.bin_dir();
    println!("Tools dir: {}", tools_dir.display());
    println!("Bin dir:   {}", bin_dir.display());
    println!();
    println!("Config keys: tools_dir / bin_dir in config.toml");
    println!("Env var:     I_RS_CODE_DIR");
    Ok(())
}
