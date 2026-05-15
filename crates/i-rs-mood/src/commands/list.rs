use crate::models::MoodRecord;
use crate::presentation::{format_table, print_mood_calendar, print_record_count, print_warning, output_list, OutputFormat};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_list(days: Option<usize>, calendar: bool, format: OutputFormat) -> Result<()> {
    let records = crate::service::list_moods(days)?;
    let records_ref: Vec<&MoodRecord> = records.iter().collect();

    if records_ref.is_empty() {
        if format.is_json() {
            let filter = days.map(|d| format!("last {d} days"));
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter.as_deref(), format));
        } else {
            print_warning("No mood records found.");
        }
        return Ok(());
    }

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            date: String,
            mood: String,
            mood_label: String,
            tags: Vec<String>,
            content: Vec<String>,
        }

        let items: Vec<ListItem> = records_ref.iter().map(|r| ListItem {
            date: r.date.format("%Y-%m-%d").to_string(),
            mood: r.mood.to_string(),
            mood_label: r.mood.label().to_string(),
            tags: r.tags.clone(),
            content: r.content.clone(),
        }).collect();

        let filter = days.map(|d| format!("last {d} days"));
        println!("{}", output_list(&items, items.len(), filter.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&records_ref);
    println!("\n{table}");

    print_record_count(records_ref.len());

    // Re-load store for stats (mood_stats needs the full data set)
    let store = crate::storage::load_store()?;
    if let Some((min_mood, max_mood, avg)) = store.mood_stats() {
        println!("\n{}", "Statistics:".bold().cyan());
        println!("  {:12} {} {}", "Best:".dimmed(), min_mood, min_mood.label());
        println!("  {:12} {} {}", "Worst:".dimmed(), max_mood, max_mood.label());
        println!("  {:12} {:.1}/5", "Average:".dimmed(), avg);
    }

    if calendar {
        let cal_days = days.unwrap_or(30);
        print_mood_calendar(&records_ref, cal_days);
    }

    Ok(())
}
