use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(date: String, mood: String, tag: Vec<String>, content: Vec<String>) -> Result<()> {
    let record = crate::service::add_mood(date, mood.clone(), tag, content)?;
    print_success(&format!(
        "✓ Mood record added: {} {}",
        record.mood,
        record.mood.label().green()
    ));
    Ok(())
}
