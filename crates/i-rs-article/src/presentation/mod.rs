use crate::models::{ArticleRow, Article};
use crate::storage::ArticleStats;
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_error, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};

pub fn format_table(articles: &[&Article]) -> String {
    let rows: Vec<ArticleRow> = articles
        .iter()
        .map(|a| ArticleRow::from_article(a))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_article_count(count: usize) {
    println!("\n{} {} articles", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_stats(stats: &ArticleStats) {
    println!();
    println!("{}", "📊 Article Statistics".bold().cyan());
    println!();
    println!("  {}: {}", "Total".dimmed(), stats.total.to_string().cyan());
    println!("  {}: {}", "Unread".dimmed(), stats.unread.to_string().cyan());
    println!("  {}: {}", "Reading".dimmed(), stats.reading.to_string().cyan());
    println!("  {}: {}", "Read".dimmed(), stats.read.to_string().cyan());
    println!();
}
