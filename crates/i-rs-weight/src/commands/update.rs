use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(date: String, weight: Option<f64>, remark: Option<Vec<String>>) -> Result<()> {
    crate::service::update_weight(date.clone(), weight, remark)?;
    print_success(&format!("✓ Record for {} updated", date.green()));
    Ok(())
}
