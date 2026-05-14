use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub purchase_date: DateTime<Utc>,
    #[serde(default)]
    pub cycle_days: Option<i64>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestByStore {
    pub entries: BTreeMap<String, Entity>,
}

impl Default for BestByStore {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl BestByStore {
    pub fn add_entry(&mut self, entry: Entity) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<Entity> {
        self.entries.remove(name)
    }

    pub fn get_entry(&self, name: &str) -> Option<&Entity> {
        self.entries.get(name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut Entity> {
        self.entries.get_mut(name)
    }

    pub fn get_all_entries(&self) -> Vec<&Entity> {
        self.entries.values().collect()
    }
}

impl Entity {
    pub fn days_until_replace(&self) -> Option<i64> {
        self.cycle_days.map(|cycle| {
            let replace_date = self.purchase_date + chrono::Duration::days(cycle);
            (replace_date - Utc::now()).num_days()
        })
    }

    pub fn is_expired(&self) -> Option<bool> {
        self.days_until_replace().map(|days| days < 0)
    }

    pub fn is_soon(&self) -> Option<bool> {
        self.days_until_replace().map(|days| days >= 0 && days <= 7)
    }
}

#[derive(Tabled, Clone)]
pub struct EntityRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "PURCHASED")]
    purchase_date: String,
    #[tabled(rename = "CYCLE")]
    cycle_days: String,
    #[tabled(rename = "REPLACE_IN")]
    replace_in: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl EntityRow {
    pub fn from_entity(entity: &Entity) -> Self {
        let status = if let Some(expired) = entity.is_expired() {
            if expired {
                "EXPIRED"
            } else if entity.is_soon().unwrap_or(false) {
                "SOON"
            } else {
                "OK"
            }
        } else {
            "NO_CYCLE"
        };

        let replace_in = entity
            .days_until_replace()
            .map(|d| {
                if d < 0 {
                    format!("{}d ago", d.abs())
                } else {
                    format!("{}d", d)
                }
            })
            .unwrap_or_else(|| "-".to_string());

        let cycle_str = entity
            .cycle_days
            .map(|d| format!("{}d", d))
            .unwrap_or_else(|| "-".to_string());

        Self {
            name: entity.name.clone(),
            purchase_date: entity.purchase_date.format("%Y-%m-%d").to_string(),
            cycle_days: cycle_str,
            replace_in,
            status: status.to_string(),
            tags: if entity.tags.is_empty() {
                "-".to_string()
            } else {
                entity.tags.join(", ")
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub name: String,
    pub purchase_date: String,
    pub cycle_days: Option<i64>,
    pub replace_in: Option<i64>,
    pub status: String,
    pub tags: Vec<String>,
}

impl From<&Entity> for ListItem {
    fn from(entity: &Entity) -> Self {
        let status = if let Some(expired) = entity.is_expired() {
            if expired {
                "EXPIRED".to_string()
            } else if entity.is_soon().unwrap_or(false) {
                "SOON".to_string()
            } else {
                "OK".to_string()
            }
        } else {
            "NO_CYCLE".to_string()
        };

        Self {
            name: entity.name.clone(),
            purchase_date: entity.purchase_date.format("%Y-%m-%d").to_string(),
            cycle_days: entity.cycle_days,
            replace_in: entity.days_until_replace(),
            status,
            tags: entity.tags.clone(),
        }
    }
}
