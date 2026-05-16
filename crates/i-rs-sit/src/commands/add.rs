use crate::models::SitEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_add(duration_minutes: i32, tag: Vec<String>, remark: Vec<String>) -> Result<()> {
    let mut store = storage::load_store()?;

    let now = Utc::now();
    let started_at = now - chrono::Duration::minutes(i64::from(duration_minutes));
    let ended_at = now;

    let entry = SitEntry::new(duration_minutes, started_at, ended_at, tag, remark);

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Recorded {} minutes of sitting",
        duration_minutes.green()
    ));

    Ok(())
}
