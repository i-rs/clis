use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(id: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.remove_record(&id).is_none() {
        print_error(&format!("No record found with ID: {}", id));
        anyhow::bail!("No record found with ID: {}", id);
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Record {} deleted", id.green()));

    Ok(())
}
