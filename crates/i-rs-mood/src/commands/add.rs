use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    date: String,
    mood: String,
    tag: Vec<String>,
    content: Vec<String>,
) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let record = crate::service::add_mood(&mut store, date, mood.clone(), tag, content)?;
    crate::storage::save_store(&store)?;
    print_success(&format!(
        "✓ Mood record added: {} {}",
        record.mood,
        record.mood.label().green()
    ));
    Ok(())
}
