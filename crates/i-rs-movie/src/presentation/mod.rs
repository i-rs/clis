use crate::models::{Movie, MovieRow, MovieStats};
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_error, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(movies: &[&Movie]) -> String {
    let rows: Vec<MovieRow> = movies
        .iter()
        .map(|m| MovieRow::from_movie(m))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_count(count: usize) {
    println!("\n{} {} movies", "Total:".dimmed(), count.to_string().cyan());
}
pub fn print_stats(stats: &MovieStats) {
    println!("\n{}", "Statistics:".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());
    println!("{} {}", "Total:".dimmed(), stats.total.to_string().green());
    println!("{} {}", "Watched:".dimmed(), stats.watched.to_string().green());
    println!("{} {}", "Unwatched:".dimmed(), stats.unwatched.to_string().green());
    if let Some(avg) = stats.avg_rating {
        println!("{} {:.2}", "Avg Rating:".dimmed(), format!("{:.2}", avg).green());
    } else {
        println!("{} {}", "Avg Rating:".dimmed(), "-".to_string());
    }
}
