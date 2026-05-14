use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(id: String) -> Result<()> {
    let mut store = storage::load_store()?;

    let record = store.entries.values().find(|r| r.id == id || r.id.starts_with(&id));

    match record {
        Some(r) => {
            let record_id = r.id.clone();
            storage::remove_entry(&mut store, &record_id);
            storage::save_store(&store)?;
            print_success(&format!("✓ Deploy record '{}' deleted", record_id.green()));
        }
        None => {
            print_error(&format!("Deploy record '{}' not found", id));
            anyhow::bail!("Record not found");
        }
    }

    Ok(())
}
