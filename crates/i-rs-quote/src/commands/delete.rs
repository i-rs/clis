use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(id: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if storage::remove_quote(&mut store, &id).is_none() {
        print_error(&format!("Quote '{}' not found", id));
        anyhow::bail!("Quote '{}' not found", id);
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Quote '{}' deleted successfully", id.green()));

    Ok(())
}
