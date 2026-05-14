use crate::models::{format_pace, RunRecord};
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::{NaiveDate, Utc};
use owo_colors::OwoColorize;
use uuid::Uuid;

pub fn handle_add(
    date: String,
    distance: f64,
    duration: f64,
    heart_rate: Option<u32>,
    weather: Option<String>,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let date = parse_date(&date)?;
    let pace = format_pace(distance, duration);

    let record = RunRecord {
        id: Uuid::new_v4().to_string(),
        date,
        distance_km: distance,
        duration_minutes: duration,
        pace: pace.clone(),
        heart_rate,
        weather,
        tags,
        remark,
        created_at: Utc::now(),
    };

    let mut store = storage::load_store()?;
    store.add_record(record);
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Run record added: {} km in {} (pace: {})",
        distance.green(),
        format_duration(duration).green(),
        pace.green()
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

fn format_duration(minutes: f64) -> String {
    let hours = (minutes / 60.0) as u32;
    let mins = (minutes % 60.0) as u32;
    let secs = ((minutes * 60.0) % 60.0) as u32;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{}:{:02}", mins, secs)
    }
}
