use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(date: String, weight: f64, remark: Vec<String>) -> Result<()> {
    crate::service::add_weight(date, weight, remark)?;
    print_success(&format!("✓ Weight record added: {} kg", weight.green()));
    Ok(())
}
