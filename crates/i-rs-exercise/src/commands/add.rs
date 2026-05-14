use crate::models::ExerciseRecord;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::validate_name;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    exercise_type: String,
    duration_minutes: u32,
    calories: Option<u32>,
    notes: Vec<String>,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    if let Err(e) = validate_name(&name) {
        anyhow::bail!("{}", e.message);
    }

    let mut store = storage::load_store()?;

    if store.records.contains_key(&name) {
        anyhow::bail!("Exercise '{}' already exists", name);
    }

    let now = Utc::now();
    let record = ExerciseRecord {
        name: name.clone(),
        exercise_type,
        duration_minutes,
        calories,
        notes,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_record(record);
    storage::save_store(&store)?;

    print_success(&format!("✓ Exercise '{}' added", name.green()));

    Ok(())
}
