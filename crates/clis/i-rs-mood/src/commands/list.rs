use crate::models::{ListItem, MoodRecord, MoodRow};
use crate::presentation::{
    OutputFormat, format_table, output_list, print_entry_count, print_mood_calendar, print_warning,
};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_list(
    days: Option<usize>,
    calendar: bool,
    limit: Option<usize>,
    offset: Option<usize>,
    format: OutputFormat,
) -> Result<()> {
    let store = crate::storage::load_store()?;
    let mut records = crate::service::list_moods(&store, days)?;
    let total = records.len();

    if limit.is_some() || offset.is_some() {
        let start = offset.unwrap_or(0).min(total);
        let end = limit.map(|l| (start + l).min(total)).unwrap_or(total);
        records = records[start..end].to_vec();
    }
    let shown = records.len();
    let records_ref: Vec<&MoodRecord> = records.iter().collect();

    if records_ref.is_empty() {
        if format.is_json() {
            let filter = days.map(|d| format!("last {d} days"));
            println!(
                "{}",
                output_list::<serde_json::Value>(&[], total, filter.as_deref(), format)
            );
        } else {
            print_warning("No mood records found.");
        }
        return Ok(());
    }

    if format.is_json() {
        let items: Vec<ListItem> = records_ref.iter().map(|r| ListItem::from(*r)).collect();
        let filter = days.map(|d| format!("last {d} days"));
        println!(
            "{}",
            output_list(&items, total, filter.as_deref(), format)
        );
        return Ok(());
    }

    let rows: Vec<MoodRow> = records_ref.iter().map(|r| MoodRow::from_record(r)).collect();
    let table = format_table(&rows);
    println!("\n{table}");

    if shown < total {
        println!(
            "  {} {}-{} / {}",
            "Showing:".dimmed(),
            offset.unwrap_or(0) + 1,
            offset.unwrap_or(0) + shown,
            total
        );
    }
    print_entry_count(shown);

    if let Some((min_mood, max_mood, avg)) = store.mood_stats() {
        println!("\n{}", "Statistics:".bold().cyan());
        println!("  {:12} {} {}", "Best:".dimmed(), min_mood, min_mood.label());
        println!(
            "  {:12} {} {}",
            "Worst:".dimmed(),
            max_mood,
            max_mood.label()
        );
        println!("  {:12} {:.1}/7", "Average:".dimmed(), avg);
    }

    if calendar {
        let cal_days = days.unwrap_or(30);
        print_mood_calendar(&records_ref, cal_days);
    }

    Ok(())
}
