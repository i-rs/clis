use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    match store.remove_entry(&name) {
        Some(_) => {
            storage::save_store(&store)?;
            print_success(&format!("✓ Gift '{}' deleted successfully", name.green()));
        }
        None => {
            anyhow::bail!("Gift '{name}' not found");
        }
    }

    Ok(())
}
