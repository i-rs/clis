use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(name: String, key_type: Option<String>, key_value: Option<String>, tag: Option<Vec<String>>, remark: Option<Vec<String>>) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::update_key(&mut store, name.clone(), key_type, key_value, tag, remark)?;
    crate::storage::save_store(&store)?;
    print_success(&format!("✓ Key '{}' updated successfully", name.green()));
    Ok(())
}
