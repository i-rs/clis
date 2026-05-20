i_rs_core::presentation!(SleepRow, "records");

use crate::models::SleepStats;

pub fn print_stats(stats: &SleepStats) {
    let style = owo_colors::Style::new().bold();
    println!("\n{}", "Sleep Statistics".style(style).cyan());
    println!(
        "{} {}",
        "Total records:".dimmed(),
        stats.total_records.to_string().cyan()
    );
    println!(
        "{} {}h",
        "Average duration:".dimmed(),
        format!("{:.1}", stats.avg_duration).green()
    );
    println!(
        "{} {}",
        "Average quality:".dimmed(),
        format!("{:.1}/5", stats.avg_quality).green()
    );
    println!(
        "{} {}h",
        "Min duration:".dimmed(),
        format!("{:.1}", stats.min_duration).yellow()
    );
    println!(
        "{} {}h",
        "Max duration:".dimmed(),
        format!("{:.1}", stats.max_duration).yellow()
    );
}
