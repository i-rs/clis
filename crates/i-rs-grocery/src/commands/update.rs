use crate::presentation::print_success;
use crate::storage;

pub fn handle_update(name: String, quantity: Option<i32>, unit: Option<String>, tags: Option<Vec<String>>, remark: Option<Vec<String>>) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    storage::update_item(&mut store, &name, quantity, unit, tags, remark)?;
    storage::save_store(&store)?;

    print_success(&format!("Item '{name}' updated successfully"));

    Ok(())
}