use crate::presentation::print_stats;
use crate::storage;
use anyhow::Result;

pub fn handle_stats() -> Result<()> {
    let store = storage::load_store()?;
    let stats = storage::get_stats(&store);
    print_stats(&stats);
    Ok(())
}
