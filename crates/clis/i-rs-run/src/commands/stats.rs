use crate::presentation::print_stats;
use crate::storage;
use anyhow::Result;

pub fn handle_stats() -> Result<()> {
    let store = storage::load_store()?;
    print_stats(&store);
    Ok(())
}
