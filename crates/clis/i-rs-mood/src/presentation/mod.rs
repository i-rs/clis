use crate::models::MoodRecord;

i_rs_core::presentation!(MoodRow, "moods", print_warning);

pub fn print_mood_calendar(records: &[&MoodRecord], days: usize) {
    println!("\n{}", "Mood Calendar:".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());
    let today = chrono::Utc::now().date_naive();
    let start_date = today - chrono::Duration::days(days as i64 - 1);
    let record_map: std::collections::HashMap<_, _> =
        records.iter().map(|r| (r.date, *r)).collect();
    let mut current = start_date;
    while current <= today {
        let mood_str = if let Some(record) = record_map.get(&current) {
            record.mood.emoji().to_string()
        } else {
            "·".dimmed().to_string()
        };
        print!("{mood_str} ");
        current += chrono::Duration::days(1);
    }
    println!();
    println!("\nLegend: 😊 Great  🙂 Good  😐 Okay  😔 Bad  😢 Terrible");
}
