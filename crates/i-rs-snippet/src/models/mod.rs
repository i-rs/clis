use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub name: String,
    pub language: String,
    pub code: Vec<String>,
    #[serde(default)]
    pub description: Vec<String>,
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
pub struct SnippetStore {
    pub snippets: std::collections::BTreeMap<String, Snippet>,
}

impl Default for SnippetStore {
    fn default() -> Self {
        Self {
            snippets: std::collections::BTreeMap::new(),
        }
    }
}

#[derive(Tabled)]
pub struct SnippetRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "LANGUAGE")]
    language: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "CREATED")]
    created_at: String,
    #[tabled(rename = "UPDATED")]
    updated_at: String,
}

impl SnippetRow {
    pub fn from_snippet(snippet: &Snippet) -> Self {
        Self {
            name: snippet.name.clone(),
            language: snippet.language.clone(),
            tags: if snippet.tags.is_empty() {
                "-".to_string()
            } else {
                snippet.tags.join(", ")
            },
            created_at: snippet.created_at.format("%Y-%m-%d %H:%M").to_string(),
            updated_at: snippet.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}
