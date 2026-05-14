use crate::models::{Habit, HabitStore};
use anyhow::Result;
use chrono::Utc;


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


i_rs_core::create_store!(HabitStore, "habit");
