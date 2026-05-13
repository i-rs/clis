use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if storage::remove_remind(&mut store, &name).is_none() {
        print_error(&format!("Remind '{}' not found", name));
        anyhow::bail!("Remind '{}' not found", name);
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Remind '{}' deleted successfully", name.green()));

    Ok(())
}
