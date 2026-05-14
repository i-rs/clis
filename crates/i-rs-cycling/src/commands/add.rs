use crate::models::CyclingRecord;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use owo_colors::OwoColorize;

pub fn handle_add(
    date: String,
    distance: f64,
    duration: u32,
    elevation: Option<f64>,
    route: Option<String>,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let date = parse_date(&date)?;

    if distance <= 0.0 {
        print_error("Distance must be greater than 0");
        anyhow::bail!("Distance must be greater than 0");
    }

    if duration == 0 {
        print_error("Duration must be greater than 0");
        anyhow::bail!("Duration must be greater than 0");
    }

    let record = CyclingRecord::new(date, distance, duration, elevation, route, tags, remark);

    let mut store = storage::load_store()?;
    store.add_record(record.clone());
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Cycling record added: {} km in {} min ({} km/h)",
        distance.green(),
        duration.to_string().green(),
        format!("{:.1}", record.avg_speed).green()
    ));

    Ok(())
}

fn parse_date(date_str: &str) -> Result<NaiveDate> {
    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }

    Err(anyhow::anyhow!(
        "Invalid date format: {}. Use YYYY-MM-DD",
        date_str
    ))
}
