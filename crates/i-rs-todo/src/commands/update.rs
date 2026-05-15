use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    title: Option<String>,
    priority: Option<String>,
    tag: Option<Vec<String>>,
    content: Option<Vec<String>>,
) -> Result<()> {
    crate::service::update_todo(name.clone(), title, priority, tag, content)?;
    print_success(&format!("✓ Todo '{}' updated", name.green()));
    Ok(())
}
