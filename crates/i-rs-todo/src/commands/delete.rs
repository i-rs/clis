use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::delete_todo(&mut store, &name)?;
    crate::storage::save_store(&store)?;
    print_success(&format!("✓ Todo '{}' deleted", name.green()));
    Ok(())
}
