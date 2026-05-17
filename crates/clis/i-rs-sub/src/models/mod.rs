use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubEntry {
    pub name: String,
    pub amount: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub next_billing_date: DateTime<Utc>,
    pub url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl SubEntry {
    pub fn days_until_next(&self) -> i64 {
        (self.next_billing_date - Utc::now()).num_days()
    }

    pub fn is_expiring_soon(&self) -> bool {
        let days = self.days_until_next();
        (0..=7).contains(&days)
    }

    pub fn is_expired(&self) -> bool {
        self.days_until_next() < 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubStore {
    pub entries: BTreeMap<String, SubEntry>,
}

impl SubStore {
    pub fn add_entry(&mut self, entry: SubEntry) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<SubEntry> {
        self.entries.remove(name)
    }

    pub fn get_entry(&self, name: &str) -> Option<&SubEntry> {
        self.entries.get(name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut SubEntry> {
        self.entries.get_mut(name)
    }
}

#[derive(Tabled)]
pub struct SubRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "AMOUNT")]
    amount: String,
    #[tabled(rename = "CYCLE")]
    billing_cycle: String,
    #[tabled(rename = "NEXT_IN")]
    next_in: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl SubRow {
    pub fn from_entry(entry: &SubEntry) -> Self {
        let days = entry.days_until_next();
        let next_str = if days < 0 {
            format!("{}d ago", days.abs())
        } else {
            format!("{days}d")
        };

        let status = if entry.is_expired() {
            "EXPIRED"
        } else if entry.is_expiring_soon() {
            "DUE_SOON"
        } else {
            "OK"
        };

        Self {
            name: entry.name.clone(),
            amount: format!("{:.2} {}", entry.amount, entry.currency),
            billing_cycle: entry.billing_cycle.clone(),
            next_in: next_str,
            status: status.to_string(),
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
    pub name: String,
    pub amount: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub next_in: i64,
    pub status: String,
    pub tags: Vec<String>,
}

impl From<&SubEntry> for ListItem {
    fn from(entry: &SubEntry) -> Self {
        let status = if entry.is_expired() {
            "EXPIRED".to_string()
        } else if entry.is_expiring_soon() {
            "DUE_SOON".to_string()
        } else {
            "OK".to_string()
        };

        Self {
            name: entry.name.clone(),
            amount: entry.amount,
            currency: entry.currency.clone(),
            billing_cycle: entry.billing_cycle.clone(),
            next_in: entry.days_until_next(),
            status,
            tags: entry.tags.clone(),
        }
    }
}
