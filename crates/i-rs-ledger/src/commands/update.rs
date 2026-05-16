use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

pub fn handle_update(
    id: String,
    date: Option<String>,
    amount: Option<f64>,
    entry_type: Option<String>,
    category: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let short_id = if id.len() >= 8 { &id[..8] } else { &id };

    let entry_mut = match store.entries.get_mut(short_id) {
        Some(e) => e,
        None => {
            anyhow::bail!("Entry '{id}' not found");
        }
    };

    if let Some(date) = date {
        entry_mut.date = parse_date(&date)?;
    }
    i_rs_core::update_field!(entry_mut.amount, amount);
    i_rs_core::update_field!(entry_mut.entry_type, entry_type);
    i_rs_core::update_field!(entry_mut.category, category);
    i_rs_core::update_field!(entry_mut.tags, tag);
    i_rs_core::update_field!(entry_mut.remark, remark);

    storage::save_store(&store)?;

    print_success(&format!("✓ Entry '{}' updated successfully", id.green()));

    Ok(())
}
