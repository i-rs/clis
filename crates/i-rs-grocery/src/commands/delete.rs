use crate::presentation::print_success;
use crate::storage;

pub fn handle_delete(name: String) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    storage::delete_item(&mut store, &name)?;
    storage::save_store(&store)?;

    print_success(&format!("Item '{}' deleted successfully", name));

    Ok(())
}