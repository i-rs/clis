use crate::presentation::print_stats;
use crate::storage;
use anyhow::Result;

pub fn handle_stats() -> Result<()> {
    let store = storage::load_store()?;

    if store.records.is_empty() {
        println!("No cycling records to show statistics.");
        return Ok(());
    }

    print_stats(&store);

    Ok(())
}
