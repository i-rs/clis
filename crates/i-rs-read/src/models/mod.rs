use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub name: String,
    pub author: String,
    pub total_pages: u32,
    #[serde(default)]
    pub current_page: u32,
    #[serde(default)]
    pub status: BookStatus,
    #[serde(default)]
    pub rating: Option<f32>,
    #[serde(default)]
    pub review: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum BookStatus {
    Reading,
    Completed,
    Paused,
    Dropped,
    #[serde(rename = "to_read")]
    #[default]
    ToRead,
}


impl Book {
    pub fn new(name: String, author: String, total_pages: u32) -> Self {
        let now = Utc::now();
        Self {
            name,
            author,
            total_pages,
            current_page: 0,
            status: BookStatus::ToRead,
            rating: None,
            review: String::new(),
            tags: Vec::new(),
            remark: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn progress_percentage(&self) -> f32 {
        if self.total_pages == 0 {
            0.0
        } else {
            (self.current_page as f32 / self.total_pages as f32) * 100.0
        }
    }
}

#[derive(Debug, Clone, Tabled)]
pub struct BookRow {
    pub name: String,
    pub author: String,
    pub pages: String,
    pub status: String,
    pub progress: String,
    #[tabled(rename = "rating")]
    pub rating: String,
}

impl BookRow {
    pub fn from_book(book: &Book) -> Self {
        Self {
            name: book.name.clone(),
            author: book.author.clone(),
            pages: format!("{}/{}", book.current_page, book.total_pages),
            status: format!("{:?}", book.status),
            progress: format!("{:.1}%", book.progress_percentage()),
            rating: book
                .rating.map_or_else(|| "-".to_string(), |r| format!("{r:.1}")),
        }
    }
}
