use crate::presentation::print_success;
use crate::storage;

pub fn handle_clear() -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    let count = storage::clear_purchased(&mut store)?;
    storage::save_store(&store)?;

    print_success(&format!("Cleared {count} purchased items"));

    Ok(())
}