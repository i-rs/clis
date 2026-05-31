use crate::config::{self, Config};

pub async fn run_config_init() -> anyhow::Result<()> {
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
            format!("{}...{}", &k[..4], &k[k.len() - 4..])
        } else {
            "****".to_string()
        }
    });
    prompt(
        "API key",
        &masked.unwrap_or_else(|| "not set".into()),
        &mut input,
    )?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        config.api_key = Some(trimmed.clone());
        if !trimmed.starts_with("$") {
            eprintln!(
                "\n  Warning: API key stored in plaintext. Consider using I_RS_CODE_API_KEY env var instead."
            );
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

pub async fn run_config_set(key: &str, value: &str) -> anyhow::Result<()> {
    let mut config = Config::load()?;
    match key {
        "provider" => config.provider = value.to_string(),
        "api_key" => config.api_key = Some(value.to_string()),
        "base_url" => config.base_url = Some(value.to_string()),
        "model" => config.model = Some(value.to_string()),
        "workspace" => config.workspace = Some(value.to_string()),
        _ => anyhow::bail!(
            "Unknown config key: {}. Valid keys: provider, api_key, base_url, model, workspace",
            key
        ),
    }
    config.save()?;
    println!("✓ {} set to {}", key, value);
    Ok(())
}
