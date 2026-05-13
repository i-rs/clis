use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if storage::remove_bookmark(&mut store, &name).is_none() {
        print_error(&format!("Bookmark '{}' not found", name));
        anyhow::bail!("Bookmark '{}' not found", name);
    }

    storage::delete_password(&name)?;
    storage::save_store(&store)?;

    print_success(&format!("✓ Bookmark '{}' deleted successfully", name.green()));

    Ok(())
}
