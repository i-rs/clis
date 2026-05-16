use crate::models::{Movie, MovieRow, MovieStats};
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_success, print_warning};
use owo_colors::OwoColorize;
pub fn format_table(movies: &[&Movie]) -> String {
    let rows: Vec<MovieRow> = movies.iter().map(|m| MovieRow::from_movie(m)).collect();
    i_rs_core::render_table(&rows)
}
pub fn print_stats(stats: &MovieStats) {
    println!("\n{}", "Statistics:".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());
    println!("{} {}", "Total:".dimmed(), stats.total.to_string().green());
    println!(
        "{} {}",
        "Watched:".dimmed(),
        stats.watched.to_string().green()
    );
    println!(
        "{} {}",
        "Unwatched:".dimmed(),
        stats.unwatched.to_string().green()
    );
    if let Some(avg) = stats.avg_rating {
        println!(
            "{} {:.2}",
            "Avg Rating:".dimmed(),
            format!("{avg:.2}").green()
        );
    } else {
        println!("{} -", "Avg Rating:".dimmed());
    }
}
