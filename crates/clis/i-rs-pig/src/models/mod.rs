use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PigEntry {
    pub id: String,
    pub food_name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub happened_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl PigEntry {
    pub fn new(
        food_name: String,
        description: Option<String>,
        tags: Vec<String>,
        remark: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            food_name,
            description,
            tags,
            remark,
            happened_at: now,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PigStore {
    pub entries: BTreeMap<String, PigEntry>,
}

#[allow(dead_code)]
impl PigStore {
    pub fn add_entry(&mut self, entry: PigEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<PigEntry> {
        self.entries.remove(id)
    }

    pub fn get_entry(&self, id: &str) -> Option<&PigEntry> {
        self.entries.get(id)
    }

    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut PigEntry> {
        self.entries.get_mut(id)
    }

    pub fn get_entries_by_date(&self, date: chrono::NaiveDate) -> Vec<&PigEntry> {
        self.entries
            .values()
            .filter(|e| e.happened_at.date_naive() == date)
            .collect()
    }
}

#[derive(Tabled)]
pub struct PigRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "FOOD")]
    food_name: String,
    #[tabled(rename = "TIME")]
    happened_at: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl PigRow {
    pub fn from_entry(entry: &PigEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            food_name: entry.food_name.clone(),
            happened_at: entry.happened_at.format("%Y-%m-%d %H:%M").to_string(),
            tags: if entry.tags.is_empty() {
                "-".to_string()
            } else {
                entry.tags.join(", ")
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub food_name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub happened_at: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&PigEntry> for ListItem {
    fn from(entry: &PigEntry) -> Self {
        Self {
            id: entry.id.clone(),
            food_name: entry.food_name.clone(),
            description: entry.description.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            happened_at: entry.happened_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

// ── IrsTool Spec ──

impl i_rs_core::IrsTool for PigStore {
    type Entity = PigEntry;
    type Row = PigRow;
    type ListItem = ListItem;

    fn tool_name() -> &'static str { "pig" }
    fn description() -> &'static str {
        "Craving tracker — log food cravings and impulsive eating"
    }
    fn capabilities() -> Vec<i_rs_core::ToolCapability> {
        vec![i_rs_core::ToolCapability::DateRange]
    }

    fn entries(&self) -> &BTreeMap<String, PigEntry> { &self.entries }
    fn entries_mut(&mut self) -> &mut BTreeMap<String, PigEntry> { &mut self.entries }
    fn entity_id(e: &PigEntry) -> String { e.id.clone() }
    fn to_row(e: &PigEntry) -> PigRow { PigRow::from_entry(e) }
    fn to_list_item(e: &PigEntry) -> ListItem { ListItem::from(e) }
}
