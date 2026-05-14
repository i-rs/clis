use crate::models::Entity;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use i_rs_core::validate_name;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    purchase_date: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if let Err(e) = validate_name(&name) {
        print_error(&e.message);
        anyhow::bail!("{}", e.message);
    }

    if store.entries.contains_key(&name) {
        print_error(&format!("Item '{}' already exists", name));
        anyhow::bail!("Item '{}' already exists", name);
    }

    let purchase = parse_datetime(&purchase_date)?;

    let now = Utc::now();
    let entity = Entity {
        name: name.clone(),
        purchase_date: purchase,
        cycle_days: None,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, entity);
    storage::save_store(&store)?;

    print_success(&format!("✓ Item '{}' added successfully", name.green()));
    println!("  {}", "Note: Use 'update' command to set replacement cycle".dimmed());

    Ok(())
}

fn parse_datetime(datetime_str: &str) -> Result<DateTime<Utc>> {
    let formats = [
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%d-%m-%Y",
        "%d/%m/%Y",
    ];

    for format in &formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(datetime_str, format) {
            return Ok(Utc.from_utc_datetime(&naive));
        }
    }

    if let Ok(naive) = chrono::NaiveDate::parse_from_str(datetime_str, "%Y-%m-%d") {
        return Ok(Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).unwrap()));
    }

    if let Ok(naive) = chrono::NaiveDate::parse_from_str(datetime_str, "%d-%m-%Y") {
        return Ok(Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).unwrap()));
    }

    if let Ok(naive) = chrono::NaiveDate::parse_from_str(datetime_str, "%d/%m/%Y") {
        return Ok(Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).unwrap()));
    }

    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", datetime_str))
}
