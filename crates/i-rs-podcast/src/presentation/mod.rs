use crate::models::{Podcast, PodcastRow, PodcastStats};
use owo_colors::OwoColorize;
use tabled::{
    settings::{Color, object::Rows, object::Segment, style::BorderColor, style::Style, themes::Colorization},
    Table,
};

pub use i_rs_core::presentation::{print_error, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

pub fn format_table(podcasts: &[&Podcast]) -> String {
    let rows: Vec<PodcastRow> = podcasts
        .iter()
        .map(|p| PodcastRow::from_podcast(p))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

#[allow(dead_code)]
pub fn print_count(count: usize) {
    println!("\n{} {} podcasts", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_stats(stats: &PodcastStats) {
    println!("\n{}", "Statistics:".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());
    println!("{} {}", "Total:".dimmed(), stats.total.to_string().green());
    println!(
        "{} {} {} {}",
        "Status:".dimmed(),
        format!("○ {}", stats.not_started).dimmed(),
        format!("◐ {}", stats.in_progress).yellow(),
        format!("● {}", stats.completed).green()
    );
    println!("{}", "─".repeat(40).dimmed());
    println!(
        "{} {}",
        "Total Duration:".dimmed(),
        format_duration_full(stats.total_duration_secs).cyan()
    );
    println!(
        "{} {}",
        "Total Listened:".dimmed(),
        format_duration_full(stats.total_listened_secs).cyan()
    );
    println!(
        "{} {:.1}%",
        "Progress:".dimmed(),
        stats.total_progress_percent().green()
    );
}

fn format_duration_full(secs: i64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
