use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetbathEntry {
    pub id: String,
    pub pet_name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub bathed_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl PetbathEntry {
    pub fn new(pet_name: String, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self { id, pet_name, bathed_at: now, tags, remark, created_at: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetbathStore {
    pub entries: BTreeMap<String, PetbathEntry>,
}

impl Default for PetbathStore {
    fn default() -> Self {
        Self { entries: BTreeMap::new() }
    }
}

impl PetbathStore {
    pub fn add_entry(&mut self, entry: PetbathEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<PetbathEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&PetbathEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct PetbathRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "PET")]
    pet_name: String,
    #[tabled(rename = "BATHED_AT")]
    bathed_at: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl PetbathRow {
    pub fn from_entry(entry: &PetbathEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            pet_name: entry.pet_name.clone(),
            bathed_at: entry.bathed_at.format("%Y-%m-%d %H:%M").to_string(),
            tags: if entry.tags.is_empty() { "-".to_string() } else { entry.tags.join(", ") },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub pet_name: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub bathed_at: String,
}

impl From<&PetbathEntry> for ListItem {
    fn from(entry: &PetbathEntry) -> Self {
        Self {
            id: entry.id.clone(),
            pet_name: entry.pet_name.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            bathed_at: entry.bathed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
