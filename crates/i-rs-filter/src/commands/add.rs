use crate::models::FilterEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    appliance_name: String,
    filter_type: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = FilterEntry::new(appliance_name.clone(), filter_type.clone(), tag, remark);

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Cleaned {} filter ({})",
        appliance_name.green(),
        filter_type.cyan()
    ));

    Ok(())
}
