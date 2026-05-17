use crate::models::FastEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_add(target_hours: i32, tag: Vec<String>, remark: Vec<String>) -> Result<()> {
    let mut store = storage::load_store()?;

    let start_time = Utc::now();
    let entry = FastEntry::new(start_time, None, target_hours, tag, remark);

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Started fasting for {} hours",
        target_hours.green()
    ));

    Ok(())
}
