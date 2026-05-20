use crate::models::{Mood, MoodRecord, MoodStore};
use anyhow::{Context, Result};
use chrono::Utc;
use i_rs_core::parse_date;

/// List mood records, optionally filtered by recent days.
pub fn list_moods(store: &MoodStore, days: Option<usize>) -> Result<Vec<MoodRecord>> {
    let mut records: Vec<MoodRecord> = if let Some(d) = days {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(d as i64);
        store
            .entries
            .values()
            .filter(|r| r.date >= cutoff)
            .cloned()
            .collect()
    } else {
        store.entries.values().cloned().collect()
    };
    records.sort_by_key(|r| r.date);
    Ok(records)
}

/// Get a single mood record by id prefix (UUID).
pub fn get_mood(store: &MoodStore, id: &str) -> Result<MoodRecord> {
    let entry = store
        .entries
        .iter()
        .find(|(k, _)| k.starts_with(id))
        .map(|(_, v)| v)
        .with_context(|| format!("No mood record found for id '{id}'"))?;
    Ok(entry.clone())
}

/// Add a mood record with auto-generated UUID.
pub fn add_mood(
    store: &mut MoodStore,
    date_str: String,
    mood: String,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<MoodRecord> {
    let date = parse_date(&date_str)?;
    let mood_parsed = parse_mood_str(&mood);
    let now = Utc::now();
    let record = MoodRecord {
        id: uuid::Uuid::new_v4().to_string(),
        date,
        mood: mood_parsed,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(record.clone());
    Ok(record)
}

/// Update a mood record by id prefix.
pub fn update_mood(
    store: &mut MoodStore,
    id: String,
    date: Option<String>,
    mood: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<MoodRecord> {
    let key = store
        .entries
        .iter()
        .find(|(k, _)| k.starts_with(&id))
        .map(|(k, _)| k.clone())
        .with_context(|| format!("No mood record found for id '{id}'"))?;

    let record = store
        .entries
        .get_mut(&key)
        .context("Unexpected: key not found after find")?;

    if let Some(d) = date {
        record.date = parse_date(&d)?;
    }
    if let Some(ref m) = mood {
        record.mood = parse_mood_str(m);
    }
    if let Some(t) = tags {
        record.tags = t;
    }
    if let Some(r) = remark {
        record.remark = r;
    }
    record.updated_at = Utc::now();

    Ok(record.clone())
}

/// Delete a mood record by id prefix.
pub fn delete_mood(store: &mut MoodStore, id: String) -> Result<MoodRecord> {
    let key = store
        .entries
        .iter()
        .find(|(k, _)| k.starts_with(&id))
        .map(|(k, _)| k.clone())
        .with_context(|| format!("No mood record found for id '{id}'"))?;
    let record = store
        .entries
        .remove(&key)
        .context("Unexpected: entry vanished after find")?;
    Ok(record)
}

/// Calculate mood statistics.
#[allow(dead_code)]
pub fn mood_stats(store: &MoodStore) -> Result<Option<(Mood, Mood, f64)>> {
    Ok(store.mood_stats())
}

fn parse_mood_str(s: &str) -> Mood {
    match s.to_lowercase().as_str() {
        "7" | "amazing" | "🤩" => Mood::Amazing,
        "6" | "great" | "😊" => Mood::Great,
        "5" | "good" | "🙂" => Mood::Good,
        "4" | "okay" | "😐" => Mood::Okay,
        "3" | "poor" | "😕" => Mood::Poor,
        "2" | "bad" | "😔" => Mood::Bad,
        "1" | "terrible" | "😢" => Mood::Terrible,
        _ => Mood::Okay,
    }
}
