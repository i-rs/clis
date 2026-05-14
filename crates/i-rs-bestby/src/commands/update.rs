use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    cycle_days: Option<i64>,
    purchase_date: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entity = match storage::get_entry_mut(&mut store, &name) {
        Some(e) => e,
        None => {
            print_error(&format!("Item '{}' not found", name));
            anyhow::bail!("Item '{}' not found", name);
        }
    };

    if let Some(cycle) = cycle_days {
        if cycle <= 0 {
            print_error("Cycle days must be greater than 0");
            anyhow::bail!("Cycle days must be greater than 0");
        }
        entity.cycle_days = Some(cycle);
        let replace_date = entity.purchase_date + chrono::Duration::days(cycle);
        let days_until = (replace_date - Utc::now()).num_days();
        if days_until < 0 {
            println!("  {}", format!("(expired {} days ago)", days_until.abs()).red());
        } else if days_until <= 7 {
            println!("  {}", format!("({} days until replacement)", days_until).yellow());
        } else {
            println!("  {}", format!("({} days until replacement)", days_until).green());
        }
    }

    if let Some(date) = purchase_date {
        entity.purchase_date = parse_datetime(&date)?;
        if let Some(cycle) = entity.cycle_days {
            let replace_date = entity.purchase_date + chrono::Duration::days(cycle);
            let days_until = (replace_date - Utc::now()).num_days();
            println!("  {}", format!("New replace date: {}, {} days", replace_date.format("%Y-%m-%d"), days_until).cyan());
        }
    }

    if let Some(tag) = tag {
        entity.tags = tag;
    }
    if let Some(remark) = remark {
        entity.remark = remark;
    }

    entity.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Item '{}' updated successfully", name.green()));

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
