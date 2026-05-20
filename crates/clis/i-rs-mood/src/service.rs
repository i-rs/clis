use crate::models::{Mood, MoodRecord, MoodStore};
use anyhow::{Context, Result};
use chrono::Utc;
use i_rs_core::parse_date;

/// List mood records, optionally filtered by recent days.
pub fn list_moods(store: &MoodStore, days: Option<usize>) -> Result<Vec<MoodRecord>> {
    let records: Vec<MoodRecord> = if let Some(d) = days {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(d as i64);
        store
            .records
            .values()
            .filter(|r| r.date >= cutoff)
            .cloned()
            .collect()
    } else {
        store.records.values().cloned().collect()
    };
    Ok(records)
}

/// Get a single mood record by date string.
pub fn get_mood(store: &MoodStore, date_str: &str) -> Result<MoodRecord> {
    let date = parse_date(date_str)?;
    store
        .records
        .get(&date)
        .cloned()
        .with_context(|| format!("No mood record found for {date}"))
}

/// Add a mood record.
pub fn add_mood(
    store: &mut MoodStore,
    date_str: String,
    mood: String,
    tags: Vec<String>,
    content: Vec<String>,
    remark: Vec<String>,
) -> Result<MoodRecord> {
    let date = parse_date(&date_str)?;

    if store.records.contains_key(&date) {
        anyhow::bail!("Mood record for {date} already exists");
    }

    let mood_parsed = parse_mood_str(&mood);
    let now = Utc::now();
    let record = MoodRecord {
        date,
        mood: mood_parsed,
        tags,
        content,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(record.clone());
    Ok(record)
}

/// Update a mood record.
pub fn update_mood(
    store: &mut MoodStore,
    date_str: String,
    mood: Option<String>,
    tags: Option<Vec<String>>,
    content: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<MoodRecord> {
    let date = parse_date(&date_str)?;

    let record = store
        .records
        .get_mut(&date)
        .with_context(|| format!("No mood record found for {date}"))?;

    if let Some(ref m) = mood {
        record.mood = parse_mood_str(m);
    }
    if let Some(t) = tags {
        record.tags = t;
    }
    if let Some(c) = content {
        record.content = c;
    }
    if let Some(r) = remark {
        record.remark = r;
    }
    record.updated_at = Utc::now();

    let updated = record.clone();
    Ok(updated)
}

/// Delete a mood record by date string.
pub fn delete_mood(store: &mut MoodStore, date_str: String) -> Result<()> {
    let date = parse_date(&date_str)?;

    if store.remove_entry(&date).is_none() {
        anyhow::bail!("No mood record found for {date}");
    }

    Ok(())
}

/// Calculate mood statistics.
#[allow(dead_code)]
pub fn mood_stats(store: &MoodStore) -> Result<Option<(Mood, Mood, f64)>> {
    Ok(store.mood_stats())
}

fn parse_mood_str(s: &str) -> Mood {
    match s.to_lowercase().as_str() {
        "5" | "great" | "😊" => Mood::Great,
        "4" | "good" | "🙂" => Mood::Good,
        "3" | "okay" | "😐" => Mood::Okay,
        "2" | "bad" | "😔" => Mood::Bad,
        "1" | "terrible" | "😢" => Mood::Terrible,
        _ => Mood::Okay,
    }
}
