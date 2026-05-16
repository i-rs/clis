use crate::models::EntityRow;
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_header, print_success};
use owo_colors::OwoColorize;
pub fn format_table(entities: &[EntityRow]) -> String {
    i_rs_core::render_table(entities)
}
pub fn print_entity_count(count: usize) {
    println!("\n{} {} items", "Total:".dimmed(), count.to_string().cyan());
}
