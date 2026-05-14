use crate::models::{TaxRecord, TaxRecordRow};
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_success, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};
pub fn format_table(entities: &[&TaxRecord]) -> String {
    let rows: Vec<TaxRecordRow> = entities
        .iter()
        .map(|e| TaxRecordRow::from_entity(e))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_entity_count(count: usize) {
    println!("\n{} {} tax records", "Total:".dimmed(), count.to_string().cyan());
}
pub fn format_stats(stats: &TaxStats) -> String {
    let mut output = String::new();
    output.push_str(&format!("\n{} {}\n", "年度统计:".cyan().bold(), stats.year.to_string().cyan()));
    output.push_str(&format!("{} {} {}\n", "  个人所得税:".dimmed(), "总计".dimmed(), format!("{:.2}", stats.personal_total).green()));
    output.push_str(&format!("{} {} {}\n", "  增值税:".dimmed(), "总计".dimmed(), format!("{:.2}", stats.vat_total).green()));
    output.push_str(&format!("{} {}\n", "  合计:".dimmed(), format!("{:.2}", stats.total).green().bold()));
    output
}
#[derive(Debug)]
pub struct TaxStats {
    pub year: i32,
    pub personal_total: f64,
    pub vat_total: f64,
    pub total: f64,
}
impl TaxStats {
    pub fn new(year: i32) -> Self {
        Self {
            year,
            personal_total: 0.0,
            vat_total: 0.0,
            total: 0.0,
        }
    }
    pub fn add(&mut self, record: &TaxRecord) {
        match record.tax_type {
            crate::models::TaxType::Personal => {
                self.personal_total += record.amount;
            }
            crate::models::TaxType::Vat => {
                self.vat_total += record.amount;
            }
        }
        self.total += record.amount;
    }
}
