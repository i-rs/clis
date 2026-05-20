use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(
    date: String,
    mood: Option<String>,
    tag: Option<Vec<String>>,
    content: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let record = crate::service::update_mood(&mut store, date.clone(), mood, tag, content, remark)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct UpdateOutput {
            date: String,
            mood: String,
            mood_label: String,
            tags: Vec<String>,
            content: Vec<String>,
            remark: Vec<String>,
        }
        let output = UpdateOutput {
            date: record.date.format("%Y-%m-%d").to_string(),
            mood: record.mood.to_string(),
            mood_label: record.mood.label().to_string(),
            tags: record.tags.clone(),
            content: record.content.clone(),
            remark: record.remark.clone(),
        };
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!("✓ Record for {} updated", date.green()));
    Ok(())
}
