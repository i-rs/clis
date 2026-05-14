use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use i_rs_core::parse_datetime;

pub fn handle_update(
    name: String,
    event_date: Option<String>,
    title: Option<String>,
    tag: Option<Vec<String>>,
    content: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let remind = match storage::get_remind_mut(&mut store, &name) {
        Some(r) => r,
        None => {
            anyhow::bail!("Remind '{name}' not found");
        }
    };

    if let Some(date) = event_date {
        remind.event_date = parse_datetime(&date)?;
    }
    if let Some(title) = title {
        remind.title = Some(title);
    }
    if let Some(tag) = tag {
        remind.tags = tag;
    }
    if let Some(content) = content {
        remind.content = content;
    }

    remind.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Remind '{}' updated successfully", name.green()));

    Ok(())
}


