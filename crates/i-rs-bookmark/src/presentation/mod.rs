use crate::models::BookmarkRow;
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub fn print_success(msg: &str) {
    println!("{}", msg.green());
}

pub fn print_header(msg: &str) {
    println!("\n{}", msg.bold().cyan());
}

pub fn print_error(msg: &str) {
    eprintln!("{}", msg.red());
}

pub fn print_warning(msg: &str) {
    println!("{}", msg.yellow());
}

pub fn format_table(bookmarks: &[&crate::models::Bookmark]) -> String {
    let rows: Vec<BookmarkRow> = bookmarks
        .iter()
        .map(|b| BookmarkRow::from_bookmark(b))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_bookmark_count(count: usize) {
    println!("\n{} {} bookmarks", "Total:".dimmed(), count.to_string().cyan());
}
