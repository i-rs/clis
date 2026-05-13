use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    event_date: Option<String>,
    title: Option<String>,
    tag: Option<Vec<String>>,
    content: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let remind = match storage::get_remind_mut(&mut store, &name) {
        Some(r) => r,
        None => {
            print_error(&format!("Remind '{}' not found", name));
            anyhow::bail!("Remind '{}' not found", name);
        }
    };

    if let Some(date) = event_date {
        remind.event_date = parse_datetime(&date)?;
    }
    if let Some(title) = title {
        remind.title = Some(title);
    }
    if let Some(tag) = tag {
        remind.tags = tag;
    }
    if let Some(content) = content {
        remind.content = content;
    }

    remind.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Remind '{}' updated successfully", name.green()));

    Ok(())
}

fn parse_datetime(datetime_str: &str) -> Result<DateTime<Utc>> {
    let formats = [
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
        "%Y/%m/%d %H:%M",
        "%Y/%m/%d",
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

    Err(anyhow::anyhow!("Invalid datetime format: {}. Use YYYY-MM-DD or YYYY-MM-DD HH:MM", datetime_str))
}
