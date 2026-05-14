use crate::presentation::{print_error, print_success};
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
        print_error(&e.message);
        anyhow::bail!("{}", e.message);
    }

    let mut store = storage::load_store()?;

    let record = match store.get_record_mut(&name) {
        Some(r) => r,
        None => {
            print_error(&format!("Exercise '{}' not found", name));
            anyhow::bail!("Exercise '{}' not found", name);
        }
    };

    if let Some(et) = exercise_type {
        record.exercise_type = et;
    }
    if let Some(dm) = duration_minutes {
        record.duration_minutes = dm;
    }
    if let Some(c) = calories {
        record.calories = c;
    }
    if let Some(n) = notes {
        record.notes = n;
    }
    if let Some(t) = tags {
        record.tags = t;
    }
    if let Some(r) = remark {
        record.remark = r;
    }

    record.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Exercise '{}' updated", name.green()));

    Ok(())
}
