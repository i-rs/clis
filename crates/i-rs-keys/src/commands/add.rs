use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(name: String, key_type: String, key_value: String, tag: Vec<String>, remark: Vec<String>) -> Result<()> {
    crate::service::add_key(name.clone(), key_type, key_value, tag, remark)?;
    print_success(&format!("✓ Key '{}' added and stored securely", name.green()));
    println!("  {}", "Value stored in OS keychain".dimmed());
    Ok(())
}
