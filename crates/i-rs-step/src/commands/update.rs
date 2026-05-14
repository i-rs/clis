use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use owo_colors::OwoColorize;

pub fn handle_update(
    date: String,
    steps: Option<i32>,
    distance: Option<f64>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = parse_date(&date)?;

    let entry = match storage::get_entry_mut(&mut store, &parsed_date) {
        Some(e) => e,
        None => {
            print_error(&format!("No record for {}", date));
            anyhow::bail!("No record for {}", date);
        }
    };

    if let Some(s) = steps {
        entry.steps = s;
    }
    if let Some(d) = distance {
        entry.distance = Some(d);
    }
    if let Some(t) = tag {
        entry.tags = t;
    }
    if let Some(r) = remark {
        entry.remark = r;
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Record for {} updated", date.green()));

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
