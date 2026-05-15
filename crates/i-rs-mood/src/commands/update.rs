use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(date: String, mood: Option<String>, tag: Option<Vec<String>>, content: Option<Vec<String>>) -> Result<()> {
    crate::service::update_mood(date.clone(), mood, tag, content)?;
    print_success(&format!("✓ Record for {} updated", date.green()));
    Ok(())
}
