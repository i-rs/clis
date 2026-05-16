use crate::models::{Checkin, Habit, HabitStore};
use anyhow::{Context, Result};
use chrono::Utc;

/// List habits, optionally filtered by tag.
pub fn list_habits(store: &HabitStore, tag: Option<String>) -> Result<Vec<Habit>> {
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
pub fn get_habit(store: &HabitStore, name: &str) -> Result<Habit> {
    store
        .get_entry(name)
        .cloned()
        .with_context(|| format!("Habit '{name}' not found"))
}

/// Add a new habit.
pub fn add_habit(
    store: &mut HabitStore,
    name: String,
    description: String,
    frequency: String,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<Habit> {
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
    Ok(habit)
}

/// Check in to a habit (add timestamp).
pub fn checkin_habit(store: &mut HabitStore, name: &str) -> Result<Habit> {
    let habit = store
        .get_entry_mut(name)
        .with_context(|| format!("Habit '{name}' not found"))?;

    let now = Utc::now();
    habit.checkins.push(Checkin { date: now });
    habit.updated_at = now;

    let updated = habit.clone();
    Ok(updated)
}

/// Update a habit.
pub fn update_habit(
    store: &mut HabitStore,
    name: String,
    description: Option<String>,
    frequency: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<Habit> {
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
    Ok(updated)
}

/// Delete a habit.
pub fn delete_habit(store: &mut HabitStore, name: &str) -> Result<()> {
    if store.remove_entry(name).is_none() {
        anyhow::bail!("Habit '{name}' not found");
    }

    Ok(())
}
