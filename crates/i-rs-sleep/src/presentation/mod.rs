use crate::models::{SleepRow, SleepStats};
use owo_colors::OwoColorize;

pub use i_rs_core::presentation::{print_header, print_success, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};

pub fn format_table(rows: &[SleepRow]) -> String {
    i_rs_core::render_table(&rows)
}

pub fn print_stats(stats: &SleepStats) {
    let style = owo_colors::Style::new().bold();
    println!("\n{}", "Sleep Statistics".style(style).cyan());
    println!("{} {}", "Total records:".dimmed(), stats.total_records.to_string().cyan());
    println!("{} {}h", "Average duration:".dimmed(), format!("{:.1}", stats.avg_duration).green());
    println!("{} {}", "Average quality:".dimmed(), format!("{:.1}/5", stats.avg_quality).green());
    println!("{} {}h", "Min duration:".dimmed(), format!("{:.1}", stats.min_duration).yellow());
    println!("{} {}h", "Max duration:".dimmed(), format!("{:.1}", stats.max_duration).yellow());
}

pub fn print_record_count(count: usize) {
    println!("\n{} {} records", "Total:".dimmed(), count.to_string().cyan());
}