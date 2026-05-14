use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

pub fn handle_update(
    date: String,
    weight: Option<f64>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let date = parse_date(&date)?;

    let mut store = storage::load_store()?;

    let record = match store.get_entry_mut(&date) {
        Some(r) => r,
        None => {
            anyhow::bail!("No record found for {}", date);
        }
    };

    if let Some(w) = weight {
        record.weight = w;
    }
    if let Some(r) = remark {
        record.remark = r;
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Record for {} updated", date.green()));

    Ok(())
}

