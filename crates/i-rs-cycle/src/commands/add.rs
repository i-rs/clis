use crate::models::CycleEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    date: String,
    event_type: String,
    symptom: Vec<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Utc::now().date_naive());

    let entry = CycleEntry::new(parsed_date, event_type.clone(), symptom, tag, remark);

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Recorded {} for {}", event_type.green(), date.cyan()));

    Ok(())
}