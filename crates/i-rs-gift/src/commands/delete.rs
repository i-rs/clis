use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    match storage::remove_gift(&mut store, &name) {
        Some(_) => {
            storage::save_store(&store)?;
            print_success(&format!("✓ Gift '{}' deleted successfully", name.green()));
        }
        None => {
            print_error(&format!("Gift '{}' not found", name));
            anyhow::bail!("Gift '{}' not found", name);
        }
    }

    Ok(())
}
