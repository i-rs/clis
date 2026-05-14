use crate::models::{MilestoneRow, SavingsGoalRow};
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

pub fn format_goals_table(goals: &[&SavingsGoalRow]) -> String {
    Table::new(goals)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn format_milestones_table(milestones: &[&MilestoneRow]) -> String {
    Table::new(milestones)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_goal_count(count: usize) {
    println!("\n{} {} goals", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_progress_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64) as usize;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}
