use crate::models::Remind;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    event_date: String,
    title: Option<String>,
    tag: Vec<String>,
    content: Vec<String>,
) -> Result<()> {
    let store = storage::load_store()?;

    if store.reminds.contains_key(&name) {
        print_error(&format!("Remind '{}' already exists", name));
        anyhow::bail!("Remind '{}' already exists", name);
    }

    let event = parse_datetime(&event_date)?;

    let now = Utc::now();
    let remind = Remind {
        name: name.clone(),
        event_date: event,
        title,
        tags: tag,
        content,
        is_done: false,
        created_at: now,
        updated_at: now,
    };

    let mut store = storage::load_store()?;
    storage::add_remind(&mut store, remind);
    storage::save_store(&store)?;

    print_success(&format!("✓ Remind '{}' added successfully", name.green()));

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
