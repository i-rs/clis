use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    amount: Option<f64>,
    frequency: Option<String>,
    start_date: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = match storage::get_entry_mut(&mut store, &name) {
        Some(e) => e,
        None => {
            print_error(&format!("Entry '{}' not found", name));
            anyhow::bail!("Entry '{}' not found", name);
        }
    };

    if let Some(amount) = amount {
        entry.amount = amount;
    }

    if let Some(frequency) = frequency {
        entry.frequency = frequency;
    }

    if let Some(start_date) = start_date {
        entry.start_date = parse_date(&start_date)?;
    }

    if let Some(tag) = tag {
        entry.tags = tag;
    }

    if let Some(remark) = remark {
        entry.remark = remark;
    }

    entry.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Entry '{}' updated successfully", name.green()));

    Ok(())
}

fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];

    for format in &formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(date_str, format) {
            return Ok(Utc.from_utc_datetime(&naive));
        }
    }

    if let Ok(naive) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).unwrap()));
    }

    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", date_str))
}
