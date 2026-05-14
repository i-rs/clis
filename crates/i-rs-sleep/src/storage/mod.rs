use crate::models::{SleepRecord, SleepStore};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dirs::config_dir;
use serde_json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn get_store_path() -> Result<PathBuf> {
    let config_dir = if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir)
    } else {
        config_dir().context("Failed to get config directory")?
    };
    Ok(config_dir.join("i-rs").join("sleep.json"))
}

pub fn load_store() -> Result<SleepStore> {
    let path = get_store_path()?;
    
    if !path.exists() {
        return Ok(SleepStore::default());
    }
    
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from: {}", path.display()))
}

pub fn save_store(store: &SleepStore) -> Result<()> {
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

pub fn add_sleep(store: &mut SleepStore, bedtime: DateTime<Utc>, wake_time: DateTime<Utc>, quality: i32, tags: Vec<String>, remark: Vec<String>) -> Result<SleepRecord> {
    let now = Utc::now();
    let record = SleepRecord {
        id: Uuid::new_v4().to_string(),
        bedtime,
        wake_time,
        quality,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    
    store.add_entry(record.clone());
    Ok(record)
}

pub fn delete_sleep(store: &mut SleepStore, id: &str) -> Result<SleepRecord> {
    store.remove_entry(id)
        .ok_or_else(|| anyhow::anyhow!("Sleep record '{}' not found", id))
}

pub fn update_sleep(
    store: &mut SleepStore,
    id: &str,
    bedtime: Option<DateTime<Utc>>,
    wake_time: Option<DateTime<Utc>>,
    quality: Option<i32>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<SleepRecord> {
    let record = store.get_entry_mut(id)
        .ok_or_else(|| anyhow::anyhow!("Sleep record '{}' not found", id))?;
    
    if let Some(b) = bedtime {
        record.bedtime = b;
    }
    if let Some(w) = wake_time {
        record.wake_time = w;
    }
    if let Some(q) = quality {
        record.quality = q;
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