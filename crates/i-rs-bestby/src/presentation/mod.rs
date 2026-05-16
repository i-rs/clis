use crate::models::EntityRow;
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_header, print_success, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(entities: &[EntityRow]) -> String {
    i_rs_core::render_table(entities)
}
pub fn print_entity_count(count: usize) {
    println!("\n{} {} items", "Total:".dimmed(), count.to_string().cyan());
}
