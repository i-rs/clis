use crate::models::MoodRecord;
use crate::presentation::{format_table, print_mood_calendar, print_record_count, print_warning};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_list(days: Option<usize>, calendar: bool) -> Result<()> {
    let store = storage::load_store()?;

    let records: Vec<&MoodRecord> = if let Some(d) = days {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(d as i64);
        store
            .records
            .values()
            .filter(|r| r.date >= cutoff)
            .collect()
    } else {
        store.get_all_records()
    };

    if records.is_empty() {
        print_warning("No mood records found.");
        return Ok(());
    }

    let table = format_table(&records);
    println!("\n{}", table);

    print_record_count(records.len());

    if let Some((min_mood, max_mood, avg)) = store.mood_stats() {
        println!("\n{}", "Statistics:".bold().cyan());
        println!("  {:12} {}", "Best:".dimmed(), format!("{} {}", min_mood, min_mood.label()));
        println!("  {:12} {}", "Worst:".dimmed(), format!("{} {}", max_mood, max_mood.label()));
        println!("  {:12} {:.1}/5", "Average:".dimmed(), avg);
    }

    if calendar {
        let cal_days = days.unwrap_or(30);
        print_mood_calendar(&records, cal_days);
    }

    Ok(())
}
