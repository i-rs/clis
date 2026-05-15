use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(name: String, title: Option<String>, tag: Vec<String>, content: Vec<String>) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::add_note(&mut store, name.clone(), title, tag, content)?;
    crate::storage::save_store(&store)?;
    print_success(&format!("✓ Note '{}' added successfully", name.green()));
    Ok(())
}
