use crate::models::WeightRecord;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

pub fn handle_add(
    date: String,
    weight: f64,
    remark: Vec<String>,
) -> Result<()> {
    let date = parse_date(&date)?;

    let mut store = storage::load_store()?;

    if store.records.contains_key(&date) {
        anyhow::bail!("Record for {} already exists", date);
    }

    let record = WeightRecord {
        date,
        weight,
        tags: Vec::new(),
        remark,
    };

    store.add_record(record);
    storage::save_store(&store)?;

    print_success(&format!("✓ Weight record added: {} kg", weight.green()));

    Ok(())
}

