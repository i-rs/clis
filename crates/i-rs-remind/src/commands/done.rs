use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_done(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    let remind = match storage::get_entry_mut(&mut store, &name) {
        Some(r) => r,
        None => {
            anyhow::bail!("Remind '{name}' not found");
        }
    };

    remind.is_done = true;
    remind.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Remind '{}' marked as done", name.green()));

    Ok(())
}
