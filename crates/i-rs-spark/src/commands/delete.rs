use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(id: String) -> Result<()> {
    let mut store = storage::load_store()?;

    let short_id = if id.len() >= 8 { &id[..8] } else { &id };

    if store.remove_entry(short_id).is_none() {
        anyhow::bail!("Spark '{id}' not found");
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Spark '{}' deleted", id.green()));

    Ok(())
}
