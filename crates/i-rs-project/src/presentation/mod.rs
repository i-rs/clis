use crate::models::{Project, ProjectRow};
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};
pub fn format_project_table(projects: &[&Project]) -> String {
    let rows: Vec<ProjectRow> = projects
        .iter()
        .map(|p| ProjectRow::from_project(p))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_project_count(count: usize) {
    println!("\n{} {} projects", "Total:".dimmed(), count.to_string().cyan());
}
