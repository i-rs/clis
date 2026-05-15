use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

mod opt_ts_seconds {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(dt) => {
                let ts = dt.timestamp();
                ts.serialize(serializer)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<i64> = Option::deserialize(deserializer)?;
        Ok(opt.and_then(|ts| DateTime::from_timestamp(ts, 0)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ReadStatus {
    #[default]
    Unread,
    Reading,
    Read,
}


impl std::fmt::Display for ReadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unread => write!(f, "unread"),
            Self::Reading => write!(f, "reading"),
            Self::Read => write!(f, "read"),
        }
    }
}

impl From<&str> for ReadStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "reading" => Self::Reading,
            "read" => Self::Read,
            _ => Self::Unread,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub name: String,
    pub title: String,
    pub url: String,
    pub source: String,
    #[serde(default)]
    pub status: ReadStatus,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
    #[serde(with = "opt_ts_seconds", default)]
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ArticleStore {
    pub articles: std::collections::BTreeMap<String, Article>,
}


#[derive(Tabled)]
pub struct ArticleRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "TITLE")]
    title: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "SOURCE")]
    source: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "CREATED")]
    created_at: String,
}

impl ArticleStore {
    pub fn add_entry(&mut self, entry: Article) {
        self.articles.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, key: &str) -> Option<Article> {
        self.articles.remove(key)
    }

    pub fn get_entry(&self, key: &str) -> Option<&Article> {
        self.articles.get(key)
    }

    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut Article> {
        self.articles.get_mut(key)
    }
}

impl ArticleRow {
    pub fn from_article(article: &Article) -> Self {
        Self {
            name: article.name.clone(),
            title: if article.title.len() > 40 {
                format!("{}...", &article.title[..37])
            } else {
                article.title.clone()
            },
            status: article.status.to_string(),
            source: article.source.clone(),
            tags: if article.tags.is_empty() {
                "-".to_string()
            } else {
                article.tags.join(", ")
            },
            created_at: article.created_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ArticleDetail {
    pub name: String,
    pub title: String,
    pub url: String,
    pub source: String,
    pub status: String,
    pub notes: Vec<String>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub read_at: Option<String>,
}

impl From<&Article> for ArticleDetail {
    fn from(article: &Article) -> Self {
        Self {
            name: article.name.clone(),
            title: article.title.clone(),
            url: article.url.clone(),
            source: article.source.clone(),
            status: article.status.to_string(),
            notes: article.notes.clone(),
            tags: article.tags.clone(),
            remark: article.remark.clone(),
            created_at: article.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: article.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            read_at: article.read_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}
