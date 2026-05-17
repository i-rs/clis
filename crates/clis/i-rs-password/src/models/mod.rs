use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(skip)]
    #[allow(dead_code)]
    pub password: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PasswordStore {
    pub entries: std::collections::BTreeMap<String, PasswordEntry>,
}

impl PasswordStore {
    pub fn add_entry(&mut self, entry: PasswordEntry) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, key: &str) -> Option<PasswordEntry> {
        self.entries.remove(key)
    }

    pub fn get_entry(&self, key: &str) -> Option<&PasswordEntry> {
        self.entries.get(key)
    }

    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut PasswordEntry> {
        self.entries.get_mut(key)
    }
}

#[derive(Tabled)]
pub struct PasswordRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "URL")]
    url: String,
    #[tabled(rename = "ACCOUNT")]
    account: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "REMARK")]
    remark: String,
    #[tabled(rename = "CREATED")]
    created_at: String,
    #[tabled(rename = "UPDATED")]
    updated_at: String,
}

impl PasswordRow {
    pub fn from_entry(entry: &PasswordEntry) -> Self {
        Self {
            name: entry.name.clone(),
            url: entry.url.clone(),
            account: entry.account.clone().unwrap_or_else(|| "-".to_string()),
            tags: if entry.tags.is_empty() {
                "-".to_string()
            } else {
                entry.tags.join(", ")
            },
            remark: if entry.remark.is_empty() {
                "-".to_string()
            } else {
                entry.remark.join(", ")
            },
            created_at: entry.created_at.format("%Y-%m-%d %H:%M").to_string(),
            updated_at: entry.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}
