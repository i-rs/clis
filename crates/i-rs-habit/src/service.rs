use crate::models::{Checkin, Habit};
use crate::storage;
use anyhow::{Context, Result};
use chrono::Utc;

/// List habits, optionally filtered by tag.
pub fn list_habits(tag: Option<String>) -> Result<Vec<Habit>> {
    let store = storage::load_store()?;
    let habits: Vec<Habit> = if let Some(ref tag_filter) = tag {
        store
            .entries
            .values()
            .filter(|h| h.tags.contains(tag_filter))
            .cloned()
            .collect()
    } else {
        store.entries.values().cloned().collect()
    };
    Ok(habits)
}

/// Get a single habit by name.
pub fn get_habit(name: &str) -> Result<Habit> {
    let store = storage::load_store()?;
    store
        .get_entry(name)
        .cloned()
        .with_context(|| format!("Habit '{name}' not found"))
}

/// Add a new habit.
pub fn add_habit(
    name: String,
    description: String,
    frequency: String,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<Habit> {
    let mut store = storage::load_store()?;

    if store.get_entry(&name).is_some() {
        anyhow::bail!("Habit '{name}' already exists");
    }

    let now = Utc::now();
    let habit = Habit {
        name,
        description,
        frequency,
        tags,
        remark,
        checkins: Vec::new(),
        created_at: now,
        updated_at: now,
    };

    store.add_entry(habit.clone());
    storage::save_store(&store)?;
    Ok(habit)
}

/// Check in to a habit (add timestamp).
pub fn checkin_habit(name: &str) -> Result<Habit> {
    let mut store = storage::load_store()?;

    let habit = store
        .get_entry_mut(name)
        .with_context(|| format!("Habit '{name}' not found"))?;

    let now = Utc::now();
    habit.checkins.push(Checkin { date: now });
    habit.updated_at = now;

    let updated = habit.clone();
    storage::save_store(&store)?;
    Ok(updated)
}

/// Update a habit.
pub fn update_habit(
    name: String,
    description: Option<String>,
    frequency: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<Habit> {
    let mut store = storage::load_store()?;

    let habit = store
        .get_entry_mut(&name)
        .with_context(|| format!("Habit '{name}' not found"))?;

    if let Some(d) = description {
        habit.description = d;
    }
    if let Some(f) = frequency {
        habit.frequency = f;
    }
    if let Some(t) = tags {
        habit.tags = t;
    }
    if let Some(r) = remark {
        habit.remark = r;
    }
    habit.updated_at = Utc::now();

    let updated = habit.clone();
    storage::save_store(&store)?;
    Ok(updated)
}

/// Delete a habit.
pub fn delete_habit(name: &str) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.remove_entry(name).is_none() {
        anyhow::bail!("Habit '{name}' not found");
    }

    storage::save_store(&store)?;
    Ok(())
}
