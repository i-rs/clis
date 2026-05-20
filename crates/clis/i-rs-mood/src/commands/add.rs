use crate::presentation::OutputFormat;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    mood: String,
    date: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let date_str = date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
    let record = crate::service::add_mood(&mut store, date_str, mood, tag, remark)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        let output = crate::models::ListItem::from(&record);
        println!("{}", crate::presentation::output_item(&output, format));
        return Ok(());
    }

    crate::presentation::print_success(&format!(
        "✓ Mood record added: {} {} (id: {})",
        record.mood,
        record.mood.label(),
        record.id[..8].to_string().dimmed()
    ));
    Ok(())
}
