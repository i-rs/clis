use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if !store.articles.contains_key(&name) {
        anyhow::bail!("Article '{name}' not found");
    }

    storage::remove_entry(&mut store, &name);
    storage::save_store(&store)?;

    print_success(&format!("✓ Article '{}' deleted successfully", name.green()));

    Ok(())
}
