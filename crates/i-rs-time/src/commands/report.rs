use crate::models::{ReportData, StatsData, ListItem};
use crate::presentation::{output_report_json, OutputFormat};
use crate::storage;
use chrono::{Duration, NaiveDate, Utc};

pub fn handle_report(start_date: Option<String>, end_date: Option<String>, days: Option<i64>, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;
    let today = Utc::now().date_naive();

    let (start, end) = if let Some(d) = days {
        let end = today;
        let start = today - Duration::days(d - 1);
        (start, end)
    } else if let (Some(start_str), Some(end_str)) = (start_date, end_date) {
        let start = NaiveDate::parse_from_str(&start_str, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid start date format. Use YYYY-MM-DD"))?;
        let end = NaiveDate::parse_from_str(&end_str, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid end date format. Use YYYY-MM-DD"))?;
        (start, end)
    } else {
        let end = today;
        let start = today - Duration::days(6);
        (start, end)
    };

    let entries: Vec<_> = store.get_entries_in_range(start, end)
        .into_iter()
        .filter(|e| e.end_time.is_some())
        .collect();

    let mut daily_map: std::collections::BTreeMap<NaiveDate, Vec<_>> = std::collections::BTreeMap::new();
    for entry in &entries {
        let date = entry.start_time.date_naive();
        daily_map.entry(date).or_default().push(*entry);
    }

    let mut daily_breakdown: Vec<StatsData> = Vec::new();
    let mut current = start;
    while current <= end {
        let day_entries = daily_map.get(&current).cloned().unwrap_or_default();
        let total_minutes: i64 = day_entries.iter().map(|e| e.duration_minutes).sum();
        let items: Vec<ListItem> = day_entries.iter().map(|e| (*e).into()).collect();

        daily_breakdown.push(StatsData {
            date: current.format("%Y-%m-%d").to_string(),
            total_minutes,
            entries_count: day_entries.len(),
            entries: items,
        });

        current += Duration::days(1);
    }

    let total_minutes: i64 = daily_breakdown.iter().map(|d| d.total_minutes).sum();
    let total_hours = total_minutes as f64 / 60.0;

    let report = ReportData {
        start_date: start.format("%Y-%m-%d").to_string(),
        end_date: end.format("%Y-%m-%d").to_string(),
        total_minutes,
        total_hours,
        entries_count: entries.len(),
        daily_breakdown,
    };

    output_report_json(&report, format);

    Ok(())
}
