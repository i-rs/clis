use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(name: String, title: Option<String>, tag: Vec<String>, content: Vec<String>) -> Result<()> {
    crate::service::add_note(name.clone(), title, tag, content)?;
    print_success(&format!("✓ Note '{}' added successfully", name.green()));
    Ok(())
}
