use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(date: String, weight: f64, remark: Vec<String>, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let record = crate::service::add_weight(&mut store, date, weight, remark)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct AddOutput {
            date: String,
            weight: f64,
            tags: Vec<String>,
            remark: Vec<String>,
        }
        let output = AddOutput {
            date: record.date.format("%Y-%m-%d").to_string(),
            weight: record.weight,
            tags: record.tags.clone(),
            remark: record.remark.clone(),
        };
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Weight record added: {} kg", weight.green()));
    Ok(())
}
