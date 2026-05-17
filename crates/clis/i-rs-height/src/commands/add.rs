use crate::models::HeightRecord;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

pub fn handle_add(
    date: String,
    height: f64,
    weight: Option<f64>,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let date = parse_date(&date)?;

    let mut store = storage::load_store()?;

    if store.records.contains_key(&date) {
        anyhow::bail!("Record for {date} already exists");
    }

    let record = HeightRecord {
        date,
        height_cm: height,
        weight_kg: weight,
        tags,
        remark,
        created_at: Utc::now(),
    };

    store.add_entry(record);
    storage::save_store(&store)?;

    print_success(&format!("✓ Height record added: {} cm", height.green()));

    Ok(())
}
