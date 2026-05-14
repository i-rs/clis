use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurEntry {
    pub name: String,
    pub amount: f64,
    pub currency: String,
    pub frequency: String,
    pub start_date: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl RecurEntry {
    pub fn days_until_next(&self) -> i64 {
        let now = Utc::now();
        let days_since_start = (now - self.start_date).num_days();
        let period_days = match self.frequency.as_str() {
            "daily" => 1,
            "weekly" => 7,
            "monthly" => 30,
            "quarterly" => 90,
            "yearly" => 365,
            _ => 30,
        };
        let periods_elapsed = days_since_start / period_days;
        let next_period = periods_elapsed + 1;
        let next_date = self.start_date + chrono::Duration::days(next_period * period_days);
        (next_date - now).num_days()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurStore {
    pub entries: BTreeMap<String, RecurEntry>,
}

impl Default for RecurStore {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl RecurStore {
    pub fn add_entry(&mut self, entry: RecurEntry) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<RecurEntry> {
        self.entries.remove(name)
    }

    pub fn get_entry(&self, name: &str) -> Option<&RecurEntry> {
        self.entries.get(name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut RecurEntry> {
        self.entries.get_mut(name)
    }
}

#[derive(Tabled)]
pub struct RecurRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "AMOUNT")]
    amount: String,
    #[tabled(rename = "FREQUENCY")]
    frequency: String,
    #[tabled(rename = "NEXT_IN")]
    next_in: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl RecurRow {
    pub fn from_entry(entry: &RecurEntry) -> Self {
        let days = entry.days_until_next();
        let next_str = if days < 0 {
            format!("{}d ago", days.abs())
        } else {
            format!("{}d", days)
        };

        Self {
            name: entry.name.clone(),
            amount: format!("{:.2} {}", entry.amount, entry.currency),
            frequency: entry.frequency.clone(),
            next_in: next_str,
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
    pub frequency: String,
    pub next_in: i64,
    pub tags: Vec<String>,
}

impl From<&RecurEntry> for ListItem {
    fn from(entry: &RecurEntry) -> Self {
        Self {
            name: entry.name.clone(),
            amount: entry.amount,
            currency: entry.currency.clone(),
            frequency: entry.frequency.clone(),
            next_in: entry.days_until_next(),
            tags: entry.tags.clone(),
        }
    }
}
