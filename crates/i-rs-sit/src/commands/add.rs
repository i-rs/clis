use crate::models::SitEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use chrono::Utc;

pub fn handle_add(duration_minutes: i32, tag: Vec<String>, remark: Vec<String>) -> Result<()> {
    let mut store = storage::load_store()?;

    let now = Utc::now();
    let started_at = now - chrono::Duration::minutes(i64::from(duration_minutes));
    let ended_at = now;

    let entry = SitEntry::new(duration_minutes, started_at, ended_at, tag, remark);

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Recorded {} minutes of sitting", duration_minutes.green()));

    Ok(())
}