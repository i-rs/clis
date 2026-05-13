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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordStore {
    pub entries: std::collections::HashMap<String, PasswordEntry>,
}

impl Default for PasswordStore {
    fn default() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
        }
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
