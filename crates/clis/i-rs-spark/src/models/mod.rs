use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkEntry {
    pub id: String,
    pub content: String,
    pub source: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl SparkEntry {
    pub fn new(
        content: String,
        source: Option<String>,
        tags: Vec<String>,
        remark: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            content,
            source,
            tags,
            remark,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SparkStore {
    pub entries: BTreeMap<String, SparkEntry>,
}

#[allow(dead_code)]
impl SparkStore {
    pub fn add_entry(&mut self, entry: SparkEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<SparkEntry> {
        self.entries.remove(id)
    }

    pub fn get_entry(&self, id: &str) -> Option<&SparkEntry> {
        self.entries.get(id)
    }

    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut SparkEntry> {
        self.entries.get_mut(id)
    }
}

#[derive(Tabled)]
pub struct SparkRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "CONTENT")]
    content: String,
    #[tabled(rename = "SOURCE")]
    source: String,
    #[tabled(rename = "TIME")]
    created_at: String,
}

impl SparkRow {
    pub fn from_entry(entry: &SparkEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            content: if entry.content.len() > 50 {
                format!("{}...", &entry.content[..50])
            } else {
                entry.content.clone()
            },
            source: entry.source.clone().unwrap_or_else(|| "-".to_string()),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub content: String,
    pub source: Option<String>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&SparkEntry> for ListItem {
    fn from(entry: &SparkEntry) -> Self {
        Self {
            id: entry.id.clone(),
            content: entry.content.clone(),
            source: entry.source.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

// ── IrsTool Spec ──

impl i_rs_core::IrsTool for SparkStore {
    type Entity = SparkEntry;
    type Row = SparkRow;
    type ListItem = ListItem;

    fn tool_name() -> &'static str {
        "spark"
    }
    fn description() -> &'static str {
        "Inspiration capture — quick notes on ideas and sparks of creativity"
    }

    fn entries(&self) -> &BTreeMap<String, SparkEntry> {
        &self.entries
    }
    fn entries_mut(&mut self) -> &mut BTreeMap<String, SparkEntry> {
        &mut self.entries
    }
    fn entity_id(e: &SparkEntry) -> String {
        e.id.clone()
    }
    fn to_row(e: &SparkEntry) -> SparkRow {
        SparkRow::from_entry(e)
    }
    fn to_list_item(e: &SparkEntry) -> ListItem {
        ListItem::from(e)
    }
}
