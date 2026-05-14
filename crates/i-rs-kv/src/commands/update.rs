use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    key: String,
    value: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = match storage::get_entry_mut(&mut store, &key) {
        Some(e) => e,
        None => {
            anyhow::bail!("Key '{key}' not found");
        }
    };

    if let Some(value) = value {
        entry.value = value;
    }

    if let Some(tag) = tag {
        entry.tags = tag;
    }

    if let Some(remark) = remark {
        entry.remark = remark;
    }

    entry.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Key '{}' updated successfully", key.green()));

    Ok(())
}
