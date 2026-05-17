use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: String,
    pub content: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuoteStore {
    pub quotes: std::collections::BTreeMap<String, Quote>,
}

impl QuoteStore {
    pub fn add_entry(&mut self, entry: Quote) {
        self.quotes.insert(entry.id.clone(), entry);
    }

    pub fn remove_entry(&mut self, key: &str) -> Option<Quote> {
        self.quotes.remove(key)
    }

    pub fn get_entry(&self, key: &str) -> Option<&Quote> {
        self.quotes.get(key)
    }

    #[allow(dead_code)]
    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut Quote> {
        self.quotes.get_mut(key)
    }
}

#[derive(Tabled)]
pub struct QuoteRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "CONTENT")]
    content: String,
    #[tabled(rename = "AUTHOR")]
    author: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "CREATED")]
    created_at: String,
}

impl QuoteRow {
    pub fn from_quote(quote: &Quote) -> Self {
        let content_preview = if quote.content.len() > 50 {
            format!("{}...", &quote.content[..50])
        } else {
            quote.content.clone()
        };

        Self {
            id: quote.id.clone(),
            content: content_preview,
            author: quote.author.clone().unwrap_or_else(|| "-".to_string()),
            tags: if quote.tags.is_empty() {
                "-".to_string()
            } else {
                quote.tags.join(", ")
            },
            created_at: quote.created_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}
