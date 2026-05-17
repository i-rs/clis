use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub date: NaiveDate,
    pub amount: f64,
    pub currency: String,
    pub entry_type: String,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerStore {
    pub entries: BTreeMap<String, LedgerEntry>,
    pub categories: Vec<String>,
}

impl Default for LedgerStore {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            categories: vec![
                "income".to_string(),
                "expense".to_string(),
                "transfer".to_string(),
            ],
        }
    }
}

#[allow(dead_code)]
impl LedgerStore {
    pub fn add_entry(&mut self, entry: LedgerEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<LedgerEntry> {
        self.entries.remove(id)
    }

    pub fn get_entry(&self, id: &str) -> Option<&LedgerEntry> {
        self.entries.get(id)
    }

    pub fn get_entries_by_date(&self, date: NaiveDate) -> Vec<&LedgerEntry> {
        self.entries.values().filter(|e| e.date == date).collect()
    }

    pub fn get_entries_by_category(&self, category: &str) -> Vec<&LedgerEntry> {
        self.entries
            .values()
            .filter(|e| e.category == category)
            .collect()
    }

    pub fn total_by_type(&self, entry_type: &str) -> f64 {
        self.entries
            .values()
            .filter(|e| e.entry_type == entry_type)
            .map(|e| e.amount)
            .sum()
    }
}

#[derive(Tabled)]
pub struct LedgerRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "TYPE")]
    entry_type: String,
    #[tabled(rename = "CATEGORY")]
    category: String,
    #[tabled(rename = "AMOUNT")]
    amount: String,
    #[tabled(rename = "REMARK")]
    remark: String,
}

impl LedgerRow {
    pub fn from_entry(entry: &LedgerEntry) -> Self {
        Self {
            id: entry.id.chars().take(8).collect(),
            date: entry.date.format("%Y-%m-%d").to_string(),
            entry_type: entry.entry_type.clone(),
            category: entry.category.clone(),
            amount: format!("{:.2} {}", entry.amount, entry.currency),
            remark: if entry.remark.is_empty() {
                "-".to_string()
            } else {
                entry.remark.join(", ")
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub date: String,
    pub amount: f64,
    pub currency: String,
    pub entry_type: String,
    pub category: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
}

impl From<&LedgerEntry> for ListItem {
    fn from(entry: &LedgerEntry) -> Self {
        Self {
            id: entry.id.clone(),
            date: entry.date.format("%Y-%m-%d").to_string(),
            amount: entry.amount,
            currency: entry.currency.clone(),
            entry_type: entry.entry_type.clone(),
            category: entry.category.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Summary {
    pub total_income: f64,
    pub total_expense: f64,
    pub balance: f64,
    pub currency: String,
}

impl From<&LedgerStore> for Summary {
    fn from(store: &LedgerStore) -> Self {
        let total_income = store.total_by_type("income");
        let total_expense = store.total_by_type("expense");
        Self {
            total_income,
            total_expense,
            balance: total_income - total_expense,
            currency: "CNY".to_string(),
        }
    }
}
