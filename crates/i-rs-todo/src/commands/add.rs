use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    title: Option<String>,
    priority: Option<String>,
    tag: Vec<String>,
    content: Vec<String>,
) -> Result<()> {
    crate::service::add_todo(name.clone(), title, priority, tag, content)?;
    print_success(&format!("✓ Todo '{}' added successfully", name.green()));
    Ok(())
}
