use crate::models::{MoodRecord, MoodRow};
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub fn print_success(msg: &str) {
    println!("{}", msg.green());
}

pub fn print_error(msg: &str) {
    eprintln!("{}", msg.red());
}

pub fn print_warning(msg: &str) {
    println!("{}", msg.yellow());
}

pub fn format_table(records: &[&MoodRecord]) -> String {
    let rows: Vec<MoodRow> = records
        .iter()
        .map(|r| MoodRow::from_record(r))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_record_count(count: usize) {
    println!("\n{} {} records", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_mood_calendar(records: &[&MoodRecord], days: usize) {
    println!("\n{}", "Mood Calendar:".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());

    let today = chrono::Utc::now().date_naive();
    let start_date = today - chrono::Duration::days(days as i64 - 1);

    let record_map: std::collections::HashMap<_, _> = records
        .iter()
        .map(|r| (r.date, *r))
        .collect();

    let mut current = start_date;
    while current <= today {
        let mood_str = if let Some(record) = record_map.get(&current) {
            record.mood.emoji().to_string()
        } else {
            "·".dimmed().to_string()
        };

        print!("{} ", mood_str);
        current += chrono::Duration::days(1);
    }
    println!();

    println!("\nLegend: 😊 Great  🙂 Good  😐 Okay  😔 Bad  😢 Terrible");
}
