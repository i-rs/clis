use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mood {
    Terrible,
    Bad,
    Poor,
    Okay,
    Good,
    Great,
    Amazing,
}

impl Mood {
    pub const fn from_level(level: u8) -> Option<Self> {
        match level {
            7 => Some(Self::Amazing),
            6 => Some(Self::Great),
            5 => Some(Self::Good),
            4 => Some(Self::Okay),
            3 => Some(Self::Poor),
            2 => Some(Self::Bad),
            1 => Some(Self::Terrible),
            _ => None,
        }
    }

    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Amazing => "🤩",
            Self::Great => "😊",
            Self::Good => "🙂",
            Self::Okay => "😐",
            Self::Poor => "😕",
            Self::Bad => "😔",
            Self::Terrible => "😢",
        }
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::Amazing => "Amazing",
            Self::Great => "Great",
            Self::Good => "Good",
            Self::Okay => "Okay",
            Self::Poor => "Poor",
            Self::Bad => "Bad",
            Self::Terrible => "Terrible",
        }
    }
}

impl std::fmt::Display for Mood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.emoji())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodRecord {
    #[serde(default)]
    pub id: String,
    pub date: NaiveDate,
    pub mood: Mood,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MoodStore {
    pub entries: BTreeMap<String, MoodRecord>,
}

impl MoodStore {
    pub fn add_entry(&mut self, record: MoodRecord) {
        self.entries.insert(record.id.clone(), record);
    }

    #[allow(dead_code)]
    pub fn remove_entry(&mut self, id: &str) -> Option<MoodRecord> {
        self.entries.remove(id)
    }

    #[allow(dead_code)]
    pub fn get_entry(&self, id: &str) -> Option<&MoodRecord> {
        self.entries.get(id)
    }

    #[allow(dead_code)]
    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut MoodRecord> {
        self.entries.get_mut(id)
    }

    pub fn mood_stats(&self) -> Option<(Mood, Mood, f64)> {
        if self.entries.is_empty() {
            return None;
        }

        let moods: Vec<u8> = self
            .entries
            .values()
            .map(|r| match r.mood {
                Mood::Amazing => 7,
                Mood::Great => 6,
                Mood::Good => 5,
                Mood::Okay => 4,
                Mood::Poor => 3,
                Mood::Bad => 2,
                Mood::Terrible => 1,
            })
            .collect();

        let min_level = *moods.iter().min().expect("non-empty entries checked above");
        let max_level = *moods.iter().max().expect("non-empty entries checked above");
        let avg = f64::from(moods.iter().sum::<u8>()) / moods.len() as f64;

        let min_mood = Mood::from_level(min_level).expect("levels always 1-7 from mood_stats");
        let max_mood = Mood::from_level(max_level).expect("levels always 1-7 from mood_stats");

        Some((min_mood, max_mood, avg))
    }
}

#[derive(Debug, Clone, Serialize, tabled::Tabled)]
pub struct MoodRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "DATE")]
    pub date: String,
    #[tabled(rename = "MOOD")]
    pub mood: String,
    #[tabled(rename = "TAGS")]
    pub tags: String,
    #[tabled(rename = "REMARK")]
    pub remark: String,
}

impl MoodRow {
    pub fn from_record(record: &MoodRecord) -> Self {
        Self {
            id: record.id[..8].to_string(),
            date: record.date.format("%Y-%m-%d").to_string(),
            mood: format!("{} {}", record.mood, record.mood.label()),
            tags: if record.tags.is_empty() {
                "-".to_string()
            } else {
                record.tags.join(", ")
            },
            remark: if record.remark.is_empty() {
                "-".to_string()
            } else {
                record.remark.join(", ")
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub date: String,
    pub mood: String,
    pub mood_label: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&MoodRecord> for ListItem {
    fn from(r: &MoodRecord) -> Self {
        Self {
            id: r.id.clone(),
            date: r.date.format("%Y-%m-%d").to_string(),
            mood: r.mood.to_string(),
            mood_label: r.mood.label().to_string(),
            tags: r.tags.clone(),
            remark: r.remark.clone(),
            created_at: r.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: r.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
