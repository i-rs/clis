use crate::models::StepEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use owo_colors::OwoColorize;

pub fn handle_add(
    steps: i32,
    date: String,
    distance: Option<f64>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = parse_date(&date)?;

    let entry = StepEntry::new(steps, distance, parsed_date, tag, remark);

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    let dist_str = distance.map(|d| format!(" ({:.1} km)", d)).unwrap_or_default();
    print_success(&format!("✓ Recorded {} steps on {}{}", steps.to_string().green(), parsed_date.format("%Y-%m-%d").to_string().cyan(), dist_str));

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
