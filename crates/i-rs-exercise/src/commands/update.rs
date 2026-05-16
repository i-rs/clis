use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::validate_name;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    exercise_type: Option<String>,
    duration_minutes: Option<u32>,
    calories: Option<Option<u32>>,
    notes: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    if let Err(e) = validate_name(&name) {
        anyhow::bail!("{}", e.message);
    }

    let mut store = storage::load_store()?;

    let record = match store.get_entry_mut(&name) {
        Some(r) => r,
        None => {
            anyhow::bail!("Exercise '{name}' not found");
        }
    };

    i_rs_core::update_field!(record.exercise_type, exercise_type);
    i_rs_core::update_field!(record.duration_minutes, duration_minutes);
    i_rs_core::update_field!(record.calories, calories);
    i_rs_core::update_field!(record.notes, notes);
    i_rs_core::update_field!(record.tags, tags);
    i_rs_core::update_field!(record.remark, remark);

    record.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Exercise '{}' updated", name.green()));

    Ok(())
}
