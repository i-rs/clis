use crate::models::{MealEntry, MealStore};
use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

/// List meal entries, optionally filtered by date.
pub fn list_meals(store: &MealStore, date: Option<NaiveDate>) -> Result<Vec<MealEntry>> {
    let entries: Vec<MealEntry> = match date {
        Some(d) => store
            .entries
            .values()
            .filter(|e| e.date == d)
            .cloned()
            .collect(),
        None => store.entries.values().cloned().collect(),
    };
    Ok(entries)
}

/// Get a meal entry by id (supports short id matching).
pub fn get_meal(store: &MealStore, id: &str) -> Result<MealEntry> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    let entry = store
        .entries
        .values()
        .find(|e| e.id.starts_with(short_id))
        .cloned()
        .with_context(|| format!("Meal '{id}' not found"))?;

    Ok(entry)
}

/// Add a new meal entry.
pub fn add_meal(
    store: &mut MealStore,
    meal_type: String,
    food_items: String,
    calories: Option<i32>,
    tags: Vec<String>,
    remark: Vec<String>,
    date: NaiveDate,
) -> Result<MealEntry> {
    let now = Utc::now();
    let entry = MealEntry {
        id: Uuid::new_v4().to_string(),
        meal_type,
        food_items,
        calories,
        tags,
        remark,
        date,
        created_at: now,
    };

    store.add_entry(entry.clone());
    Ok(entry)
}

/// Update a meal entry.
pub fn update_meal(
    store: &mut MealStore,
    id: &str,
    meal_type: Option<String>,
    food_items: Option<String>,
    calories: Option<Option<i32>>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    date: Option<NaiveDate>,
) -> Result<MealEntry> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    let entry = store
        .entries
        .values_mut()
        .find(|e| e.id.starts_with(short_id))
        .with_context(|| format!("Meal '{id}' not found"))?;

    if let Some(t) = meal_type {
        entry.meal_type = t;
    }
    if let Some(f) = food_items {
        entry.food_items = f;
    }
    if let Some(c) = calories {
        entry.calories = c;
    }
    if let Some(t) = tags {
        entry.tags = t;
    }
    if let Some(r) = remark {
        entry.remark = r;
    }
    if let Some(d) = date {
        entry.date = d;
    }

    Ok(entry.clone())
}

/// Delete a meal entry by id (supports short id matching).
pub fn delete_meal(store: &mut MealStore, id: &str) -> Result<()> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    let key = store
        .entries
        .iter()
        .find(|(_, e)| e.id.starts_with(short_id))
        .map(|(k, _)| k.clone())
        .with_context(|| format!("Meal '{id}' not found"))?;

    store.remove_entry(&key);
    Ok(())
}
