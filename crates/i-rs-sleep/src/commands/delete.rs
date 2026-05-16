use crate::presentation::print_success;
use crate::storage;

pub fn handle_delete(id: String) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    storage::delete_sleep(&mut store, &id)?;
    storage::save_store(&store)?;

    print_success(&format!("Sleep record '{id}' deleted successfully"));

    Ok(())
}
