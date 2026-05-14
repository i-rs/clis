use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    key_type: Option<String>,
    key_value: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = match storage::get_entry_mut(&mut store, &name) {
        Some(e) => e,
        None => {
            anyhow::bail!("Key '{}' not found", name);
        }
    };

    if let Some(key_type) = key_type {
        entry.key_type = key_type;
    }

    if let Some(key_value) = key_value {
        storage::store_key(&name, &key_value)?;
        println!("{}", "Value updated and stored in keychain".green());
    }

    if let Some(tag) = tag {
        entry.tags = tag;
    }

    if let Some(remark) = remark {
        entry.remark = remark;
    }

    entry.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Key '{}' updated successfully", name.green()));

    Ok(())
}
