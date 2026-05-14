use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.remove_todo(&name).is_none() {
        print_error(&format!("Todo '{}' not found", name));
        anyhow::bail!("Todo '{}' not found", name);
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Todo '{}' deleted", name.green()));

    Ok(())
}
