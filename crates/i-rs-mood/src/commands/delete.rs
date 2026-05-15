use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(date: String) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::delete_mood(&mut store, date.clone())?;
    crate::storage::save_store(&store)?;
    print_success(&format!("✓ Record for {} deleted", date.green()));
    Ok(())
}
