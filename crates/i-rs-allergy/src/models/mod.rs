use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllergyEntry {
    pub id: String,
    pub allergen: String,
    pub severity: String,
    pub symptoms: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub happened_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl AllergyEntry {
    pub fn new(allergen: String, severity: String, symptoms: Vec<String>, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self { id, allergen, severity, symptoms, tags, remark, happened_at: now, created_at: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllergyStore {
    pub entries: BTreeMap<String, AllergyEntry>,
}

impl Default for AllergyStore {
    fn default() -> Self {
        Self { entries: BTreeMap::new() }
    }
}

impl AllergyStore {
    pub fn add_entry(&mut self, entry: AllergyEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<AllergyEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&AllergyEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct AllergyRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "ALLERGEN")]
    allergen: String,
    #[tabled(rename = "SEVERITY")]
    severity: String,
    #[tabled(rename = "TIME")]
    happened_at: String,
}

impl AllergyRow {
    pub fn from_entry(entry: &AllergyEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            allergen: entry.allergen.clone(),
            severity: entry.severity.clone(),
            happened_at: entry.happened_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub allergen: String,
    pub severity: String,
    pub symptoms: Vec<String>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub happened_at: String,
}

impl From<&AllergyEntry> for ListItem {
    fn from(entry: &AllergyEntry) -> Self {
        Self {
            id: entry.id.clone(),
            allergen: entry.allergen.clone(),
            severity: entry.severity.clone(),
            symptoms: entry.symptoms.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            happened_at: entry.happened_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
