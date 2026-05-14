use crate::models::{StatsData, ListItem};
use crate::presentation::{output_stats_json, OutputFormat};
use crate::storage;
use chrono::{Datelike, NaiveDate, Utc};

pub fn handle_stats(period: String, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;
    let today = Utc::now().date_naive();

    let target_date = match period.as_str() {
        "today" => today,
        "yesterday" => today - chrono::Duration::days(1),
        "week" => {
            let week_stats = calculate_week_stats(&store, today)?;
            output_stats_json(&week_stats, format);
            return Ok(());
        }
        _ => {
            anyhow::bail!("Invalid period: {}", period);
        }
    };

    let entries: Vec<_> = store.get_entries_by_date(target_date)
        .into_iter()
        .filter(|e| e.end_time.is_some())
        .collect();

    let total_minutes: i64 = entries.iter().map(|e| e.duration_minutes).sum();
    let items: Vec<ListItem> = entries.iter().map(|e| (*e).into()).collect();

    let stats = StatsData {
        date: target_date.format("%Y-%m-%d").to_string(),
        total_minutes,
        entries_count: entries.len(),
        entries: items,
    };

    output_stats_json(&stats, format);

    Ok(())
}

fn calculate_week_stats(store: &crate::models::TimeStore, today: NaiveDate) -> anyhow::Result<StatsData> {
    let start_of_week = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let end_of_week = start_of_week + chrono::Duration::days(6);

    let entries: Vec<_> = store.get_entries_in_range(start_of_week, end_of_week)
        .into_iter()
        .filter(|e| e.end_time.is_some())
        .collect();

    let total_minutes: i64 = entries.iter().map(|e| e.duration_minutes).sum();
    let items: Vec<ListItem> = entries.iter().map(|e| (*e).into()).collect();

    Ok(StatsData {
        date: format!("{} to {}", start_of_week.format("%Y-%m-%d"), end_of_week.format("%Y-%m-%d")),
        total_minutes,
        entries_count: entries.len(),
        entries: items,
    })
}
