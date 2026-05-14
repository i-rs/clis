use crate::models::TickEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use owo_colors::OwoColorize;

pub fn handle_add(
    task_name: String,
    duration_seconds: i64,
    started_at: Option<String>,
    description: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let end = Utc::now();
    let start = if let Some(s) = started_at {
        parse_datetime(&s)?
    } else {
        end - chrono::Duration::seconds(duration_seconds)
    };

    let entry = TickEntry::new(task_name.clone(), duration_seconds, description, tag, remark, start, end);

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    let duration_str = format_duration(duration_seconds);
    let total_str = format!("{}h {}m {}s",
        duration_seconds / 3600,
        (duration_seconds % 3600) / 60,
        duration_seconds % 60);
    print_success(&format!("✓ Recorded {} for '{}' ({} total)", task_name.green(), duration_str.cyan(), total_str));

    Ok(())
}

fn parse_datetime(datetime_str: &str) -> Result<DateTime<Utc>> {
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
    ];

    for format in &formats {
        if let Ok(dt) = chrono::DateTime::parse_from_str(datetime_str, format) {
            return Ok(dt.with_timezone(&Utc));
        }
    }

    if let Ok(naive) = chrono::NaiveDate::parse_from_str(datetime_str, "%Y-%m-%d") {
        return Ok(Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).expect("0:00:00 is always valid")));
    }

    Err(anyhow::anyhow!("Invalid datetime format: {datetime_str}"))
}

fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}
