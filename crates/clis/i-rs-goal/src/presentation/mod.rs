use crate::models::{MilestoneRow, SavingsGoalRow};
use owo_colors::OwoColorize;

pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success, print_warning};

pub fn format_goals_table(goals: &[&SavingsGoalRow]) -> String {
    i_rs_core::render_table(goals)
}

pub fn format_milestones_table(milestones: &[&MilestoneRow]) -> String {
    i_rs_core::render_table(milestones)
}

pub fn print_goal_count(count: usize) {
    println!("\n{} {} goals", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_progress_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64) as usize;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}
