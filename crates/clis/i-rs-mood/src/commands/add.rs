use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    date: String,
    mood: String,
    tag: Vec<String>,
    content: Vec<String>,
    remark: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let record = crate::service::add_mood(&mut store, date, mood.clone(), tag, content, remark)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct AddOutput {
            date: String,
            mood: String,
            mood_label: String,
            tags: Vec<String>,
            content: Vec<String>,
            remark: Vec<String>,
        }
        let output = AddOutput {
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

    print_success(&format!(
        "✓ Mood record added: {} {}",
        record.mood,
        record.mood.label().green()
    ));
    Ok(())
}
