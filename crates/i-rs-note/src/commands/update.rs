use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(name: String, title: Option<String>, tag: Option<Vec<String>>, content: Option<Vec<String>>) -> Result<()> {
    crate::service::update_note(name.clone(), title, tag, content)?;
    print_success(&format!("✓ Note '{}' updated successfully", name.green()));
    Ok(())
}
