pub mod output;

use owo_colors::OwoColorize;
pub use output::{OutputFormat, output_list, output_item, output_error};

pub fn print_error(msg: &str) {
    eprintln!("{}", format!("Error: {}", msg).red());
}

pub fn print_success(msg: &str) {
    println!("{}", msg.green());
}

pub fn print_header(msg: &str) {
    println!("{}", msg.bold().cyan());
}

pub fn print_warning(msg: &str) {
    println!("{}", msg.yellow());
}