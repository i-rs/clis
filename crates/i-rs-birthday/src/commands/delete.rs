use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if storage::remove_birthday(&mut store, &name).is_none() {
        anyhow::bail!("Birthday '{name}' not found");
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Birthday '{}' deleted successfully", name.green()));

    Ok(())
}
