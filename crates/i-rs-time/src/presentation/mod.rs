use crate::models::{TimeEntryRow, StatsData, ReportData};
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_error, print_header, print_success, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};

pub fn format_table(rows: &[TimeEntryRow]) -> String {
    Table::new(rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_entry_count(count: usize) {
    println!("\n{} {} entries", "Total:".dimmed(), count.to_string().cyan());
}

pub fn format_minutes(minutes: i64) -> String {
    if minutes >= 60 {
        let hours = minutes / 60;
        let mins = minutes % 60;
        if mins == 0 {
            format!("{}h", hours)
        } else {
            format!("{}h {}m", hours, mins)
        }
    } else {
        format!("{}m", minutes)
    }
}

pub fn print_stats(stats: &StatsData) {
    println!();
    print_header(&format!("Stats for {}", stats.date));
    println!("  {} {}", "Total time:".cyan(), format_minutes(stats.total_minutes).green());
    println!("  {} {}", "Entries:".cyan(), stats.entries_count.to_string().green());
    println!();
}

pub fn print_report(report: &ReportData) {
    println!();
    print_header(&format!("Report: {} to {}", report.start_date, report.end_date));
    println!("  {} {}", "Total time:".cyan(), format_minutes(report.total_minutes).green());
    println!("  {} {:.1} hours", "Hours:".cyan(), report.total_hours.green());
    println!("  {} {}", "Entries:".cyan(), report.entries_count.to_string().green());
    println!();
    print_header("Daily Breakdown");
    for daily in &report.daily_breakdown {
        println!("  {} - {}", daily.date.cyan(), format_minutes(daily.total_minutes).green());
    }
    println!();
}

pub fn output_stats_json(stats: &StatsData, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(stats).expect("stats serialization must succeed"));
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_stats(stats);
        }
    }
}

pub fn output_report_json(report: &ReportData, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(report).expect("report serialization must succeed"));
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_report(report);
        }
    }
}
