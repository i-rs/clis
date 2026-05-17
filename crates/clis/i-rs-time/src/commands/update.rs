use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    id: String,
    name: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = match store.get_entry_mut(&id) {
        Some(e) => e,
        None => anyhow::bail!("Entry '{id}' not found"),
    };

    i_rs_core::update_field!(entry.name, name);
    i_rs_core::update_field!(entry.tags, tag);
    i_rs_core::update_field!(entry.remark, remark);

    entry.updated_at = Utc::now();
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Time entry '{}' updated successfully",
        id.green()
    ));
    Ok(())
}
