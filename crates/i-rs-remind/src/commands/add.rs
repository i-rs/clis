use crate::models::Remind;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use i_rs_core::parse_datetime;

pub fn handle_add(
    name: String,
    event_date: String,
    title: Option<String>,
    tag: Vec<String>,
    content: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.reminds.contains_key(&name) {
        anyhow::bail!("Remind '{name}' already exists");
    }

    let event = parse_datetime(&event_date)?;

    let now = Utc::now();
    let remind = Remind {
        name: name.clone(),
        event_date: event,
        title,
        tags: tag,
        content,
        remark: Vec::new(),
        is_done: false,
        created_at: now,
        updated_at: now,
    };

    storage::add_remind(&mut store, remind);
    storage::save_store(&store)?;

    print_success(&format!("✓ Remind '{}' added successfully", name.green()));

    Ok(())
}


