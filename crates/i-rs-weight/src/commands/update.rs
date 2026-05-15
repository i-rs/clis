use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(date: String, weight: Option<f64>, remark: Option<Vec<String>>) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::update_weight(&mut store, date.clone(), weight, remark)?;
    crate::storage::save_store(&store)?;
    print_success(&format!("✓ Record for {} updated", date.green()));
    Ok(())
}
