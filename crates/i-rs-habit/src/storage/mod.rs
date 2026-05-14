use crate::models::{Habit, HabitStore};
use anyhow::{Context, Result};
use chrono::Utc;
use dirs::config_dir;
use serde_json;
use std::fs;
use std::path::PathBuf;

fn get_store_path() -> Result<PathBuf> {
    let config_dir = if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir)
    } else {
        config_dir().context("Failed to get config directory")?
    };
    Ok(config_dir.join("i-rs").join("habits.json"))
}

pub fn load_store() -> Result<HabitStore> {
    let path = get_store_path()?;
    
    if !path.exists() {
        return Ok(HabitStore::default());
    }
    
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from: {}", path.display()))
}

pub fn save_store(store: &HabitStore) -> Result<()> {
    let path = get_store_path()?;
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    
    let content = serde_json::to_string_pretty(store)
        .context("Failed to serialize store")?;
    
    fs::write(&path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))
}

pub fn add_habit(store: &mut HabitStore, name: String, description: String, frequency: String, tags: Vec<String>, remark: Vec<String>) -> Result<Habit> {
    let now = Utc::now();
    let habit = Habit {
        name: name.clone(),
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

pub fn delete_habit(store: &mut HabitStore, name: &str) -> Result<Habit> {
    store.remove_entry(name)
        .ok_or_else(|| anyhow::anyhow!("Habit '{}' not found", name))
}

pub fn checkin_habit(store: &mut HabitStore, name: &str) -> Result<Habit> {
    let habit = store.get_entry_mut(name)
        .ok_or_else(|| anyhow::anyhow!("Habit '{}' not found", name))?;
    
    let now = Utc::now();
    let today_checkin = Checkin { date: now };
    habit.checkins.push(today_checkin);
    habit.updated_at = now;
    
    Ok(habit.clone())
}

pub fn update_habit(
    store: &mut HabitStore,
    name: &str,
    description: Option<String>,
    frequency: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<Habit> {
    let habit = store.get_entry_mut(name)
        .ok_or_else(|| anyhow::anyhow!("Habit '{}' not found", name))?;
    
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
    
    Ok(habit.clone())
}

use crate::models::Checkin;