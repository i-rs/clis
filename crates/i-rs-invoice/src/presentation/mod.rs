use crate::models::{Invoice, InvoiceRow};
pub use i_rs_core::presentation::output::{output_error, output_item, output_list};
pub use i_rs_core::presentation::{
    OutputFormat, print_error, print_header, print_success, print_warning,
};
use owo_colors::OwoColorize;
pub fn format_table(invoices: &[&Invoice]) -> String {
    let rows: Vec<InvoiceRow> = invoices
        .iter()
        .map(|inv| InvoiceRow::from_invoice(inv))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_invoice_count(count: usize) {
    println!(
        "\n{} {} invoices",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
pub fn print_stats(
    total: f64,
    reimbursed: f64,
    unreimbursed: f64,
    count: usize,
    reimbursed_count: usize,
) {
    println!("\n{}", "=== Invoice Statistics ===".cyan().bold());
    println!(
        "{} Total amount: {:.2}",
        "Total:".dimmed(),
        total.to_string().green()
    );
    println!(
        "{} Reimbursed: {:.2} ({} invoices)",
        "Total:".dimmed(),
        reimbursed.to_string().green(),
        reimbursed_count
    );
    println!(
        "{} Unreimbursed: {:.2} ({} invoices)",
        "Total:".dimmed(),
        unreimbursed.to_string().yellow(),
        count - reimbursed_count
    );
}
