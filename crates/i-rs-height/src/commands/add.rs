use crate::models::HeightRecord;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{NaiveDate, Utc};
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
        print_error(&format!("Record for {} already exists. Use update command instead.", date));
        anyhow::bail!("Record for {} already exists", date);
    }

    let record = HeightRecord {
        date,
        height_cm: height,
        weight_kg: weight,
        tags,
        remark,
        created_at: Utc::now(),
    };

    store.add_record(record);
    storage::save_store(&store)?;

    print_success(&format!("✓ Height record added: {} cm", height.green()));

    Ok(())
}

fn parse_date(date_str: &str) -> Result<NaiveDate> {
    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }

    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", date_str))
}
