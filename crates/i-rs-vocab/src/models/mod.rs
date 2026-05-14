use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VocabStatus {
    #[default]
    New,
    Learning,
    Mastered,
}

impl VocabStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "new" => Some(VocabStatus::New),
            "learning" | "learn" => Some(VocabStatus::Learning),
            "mastered" | "master" => Some(VocabStatus::Mastered),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            VocabStatus::New => "New",
            VocabStatus::Learning => "Learning",
            VocabStatus::Mastered => "Mastered",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            VocabStatus::New => "🆕",
            VocabStatus::Learning => "📖",
            VocabStatus::Mastered => "✅",
        }
    }
}

impl std::fmt::Display for VocabStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabWord {
    pub word: String,
    pub definition: String,
    #[serde(default)]
    pub example: Vec<String>,
    #[serde(default)]
    pub status: VocabStatus,
    #[serde(default)]
    pub review_count: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabStore {
    pub words: BTreeMap<String, VocabWord>,
}

impl Default for VocabStore {
    fn default() -> Self {
        Self {
            words: BTreeMap::new(),
        }
    }
}

impl VocabStore {
    pub fn add_entry(&mut self, word: VocabWord) {
        self.words.insert(word.word.clone(), word);
    }

    pub fn remove_entry(&mut self, word_key: &str) -> Option<VocabWord> {
        self.words.remove(word_key)
    }

    pub fn get_entry(&self, word_key: &str) -> Option<&VocabWord> {
        self.words.get(word_key)
    }

    pub fn get_entry_mut(&mut self, word_key: &str) -> Option<&mut VocabWord> {
        self.words.get_mut(word_key)
    }

    pub fn get_all_words(&self) -> Vec<&VocabWord> {
        self.words.values().collect()
    }

    pub fn filter_by_status(&self, status: VocabStatus) -> Vec<&VocabWord> {
        self.words
            .values()
            .filter(|w| w.status == status)
            .collect()
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&VocabWord> {
        self.words
            .values()
            .filter(|w| w.tags.iter().any(|t| t.to_lowercase() == tag.to_lowercase()))
            .collect()
    }

    pub fn get_stats(&self) -> VocabStats {
        let total = self.words.len();
        let new_count = self.words.values().filter(|w| w.status == VocabStatus::New).count();
        let learning_count = self.words.values().filter(|w| w.status == VocabStatus::Learning).count();
        let mastered_count = self.words.values().filter(|w| w.status == VocabStatus::Mastered).count();
        let total_reviews: u32 = self.words.values().map(|w| w.review_count).sum();

        VocabStats {
            total,
            new_count,
            learning_count,
            mastered_count,
            total_reviews,
        }
    }

    pub fn get_words_for_quiz(&self, limit: usize) -> Vec<&VocabWord> {
        let mut learning_words: Vec<&VocabWord> = self.words
            .values()
            .filter(|w| w.status != VocabStatus::Mastered)
            .collect();

        learning_words.sort_by(|a, b| a.review_count.cmp(&b.review_count));
        learning_words.truncate(limit);
        learning_words
    }
}

#[derive(Debug, Clone)]
pub struct VocabStats {
    pub total: usize,
    pub new_count: usize,
    pub learning_count: usize,
    pub mastered_count: usize,
    pub total_reviews: u32,
}

#[derive(Tabled)]
pub struct VocabRow {
    #[tabled(rename = "WORD")]
    word: String,
    #[tabled(rename = "DEFINITION")]
    definition: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "REVIEWS")]
    reviews: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl VocabRow {
    pub fn from_word(word: &VocabWord) -> Self {
        Self {
            word: word.word.clone(),
            definition: if word.definition.len() > 40 {
                format!("{}...", &word.definition[..37])
            } else {
                word.definition.clone()
            },
            status: format!("{} {}", word.status.emoji(), word.status.label()),
            reviews: word.review_count.to_string(),
            tags: if word.tags.is_empty() {
                "-".to_string()
            } else {
                word.tags.join(", ")
            },
        }
    }
}
