use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if storage::remove_domain(&mut store, &name).is_none() {
        print_error(&format!("Domain '{}' not found", name));
        anyhow::bail!("Domain '{}' not found", name);
    }

    storage::delete_password(&name)?;
    storage::save_store(&store)?;

    print_success(&format!("✓ Domain '{}' deleted successfully", name.green()));

    Ok(())
}
