use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(date: String, weight: Option<f64>, remark: Option<Vec<String>>, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let record = crate::service::update_weight(&mut store, date.clone(), weight, remark)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct UpdateOutput {
            date: String,
            weight: f64,
            tags: Vec<String>,
            remark: Vec<String>,
        }
        let output = UpdateOutput {
            date: record.date.format("%Y-%m-%d").to_string(),
            weight: record.weight,
            tags: record.tags.clone(),
            remark: record.remark.clone(),
        };
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Record for {} updated", date.green()));
    Ok(())
}
