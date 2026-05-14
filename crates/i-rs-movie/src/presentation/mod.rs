use crate::models::{Movie, MovieRow, MovieStats};
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_error, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

pub fn format_table(movies: &[&Movie]) -> String {
    let rows: Vec<MovieRow> = movies
        .iter()
        .map(|m| MovieRow::from_movie(m))
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
