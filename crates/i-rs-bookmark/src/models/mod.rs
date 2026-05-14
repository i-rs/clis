use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(skip)]
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
pub struct BookmarkStore {
    pub bookmarks: std::collections::HashMap<String, Bookmark>,
}

impl Default for BookmarkStore {
    fn default() -> Self {
        Self {
            bookmarks: std::collections::HashMap::new(),
        }
    }
}

#[derive(Tabled)]
pub struct BookmarkRow {
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

impl BookmarkRow {
    pub fn from_bookmark(bookmark: &Bookmark) -> Self {
        Self {
            name: bookmark.name.clone(),
            url: bookmark.url.clone(),
            account: bookmark.account.clone().unwrap_or_else(|| "-".to_string()),
            tags: if bookmark.tags.is_empty() {
                "-".to_string()
            } else {
                bookmark.tags.join(", ")
            },
            remark: if bookmark.remark.is_empty() {
                "-".to_string()
            } else {
                bookmark.remark.join(", ")
            },
            created_at: bookmark.created_at.format("%Y-%m-%d %H:%M").to_string(),
            updated_at: bookmark.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}
