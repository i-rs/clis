use crate::presentation::print_success;
use crate::storage;

pub fn handle_update(
    name: String,
    phone: Option<String>,
    email: Option<String>,
    relationship: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    storage::update_contact(&mut store, &name, phone, email, relationship, tags, remark)?;
    storage::save_store(&store)?;

    print_success(&format!("Contact '{name}' updated successfully"));

    Ok(())
}
