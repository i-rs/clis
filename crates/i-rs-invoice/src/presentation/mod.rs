use crate::models::{Invoice, InvoiceRow};
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

pub fn format_table(invoices: &[&Invoice]) -> String {
    let rows: Vec<InvoiceRow> = invoices
        .iter()
        .map(|inv| InvoiceRow::from_invoice(inv))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_invoice_count(count: usize) {
    println!("\n{} {} invoices", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_stats(total: f64, reimbursed: f64, unreimbursed: f64, count: usize, reimbursed_count: usize) {
    println!("\n{}", "=== Invoice Statistics ===".cyan().bold());
    println!("{} Total amount: {:.2}", "Total:".dimmed(), total.to_string().green());
    println!("{} Reimbursed: {:.2} ({} invoices)", "Total:".dimmed(), reimbursed.to_string().green(), reimbursed_count);
    println!("{} Unreimbursed: {:.2} ({} invoices)", "Total:".dimmed(), unreimbursed.to_string().yellow(), count - reimbursed_count);
}
