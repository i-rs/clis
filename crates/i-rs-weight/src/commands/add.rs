use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(date: String, weight: f64, remark: Vec<String>) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::add_weight(&mut store, date, weight, remark)?;
    crate::storage::save_store(&store)?;
    print_success(&format!("✓ Weight record added: {} kg", weight.green()));
    Ok(())
}
