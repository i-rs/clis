use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mood {
    Great,
    Good,
    Okay,
    Bad,
    Terrible,
}

impl Mood {
    pub fn from_level(level: u8) -> Option<Self> {
        match level {
            5 => Some(Mood::Great),
            4 => Some(Mood::Good),
            3 => Some(Mood::Okay),
            2 => Some(Mood::Bad),
            1 => Some(Mood::Terrible),
            _ => None,
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Mood::Great => "😊",
            Mood::Good => "🙂",
            Mood::Okay => "😐",
            Mood::Bad => "😔",
            Mood::Terrible => "😢",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Mood::Great => "Great",
            Mood::Good => "Good",
            Mood::Okay => "Okay",
            Mood::Bad => "Bad",
            Mood::Terrible => "Terrible",
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
    pub date: NaiveDate,
    pub mood: Mood,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub content: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodStore {
    pub records: BTreeMap<NaiveDate, MoodRecord>,
}

impl Default for MoodStore {
    fn default() -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }
}

impl MoodStore {
    pub fn add_record(&mut self, record: MoodRecord) {
        self.records.insert(record.date, record);
    }

    pub fn remove_record(&mut self, date: &NaiveDate) -> Option<MoodRecord> {
        self.records.remove(date)
    }

    pub fn get_record(&self, date: &NaiveDate) -> Option<&MoodRecord> {
        self.records.get(date)
    }

    pub fn get_recent_records(&self, days: usize) -> Vec<&MoodRecord> {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(days as i64);
        self.records
            .values()
            .filter(|r| r.date >= cutoff)
            .collect()
    }

    pub fn get_all_records(&self) -> Vec<&MoodRecord> {
        self.records.values().collect()
    }

    pub fn mood_stats(&self) -> Option<(Mood, Mood, f64)> {
        if self.records.is_empty() {
            return None;
        }

        let moods: Vec<u8> = self.records.values().map(|r| {
            match r.mood {
                Mood::Great => 5,
                Mood::Good => 4,
                Mood::Okay => 3,
                Mood::Bad => 2,
                Mood::Terrible => 1,
            }
        }).collect();

        let min_level = *moods.iter().min().expect("non-empty records checked above");
        let max_level = *moods.iter().max().expect("non-empty records checked above");
        let avg = moods.iter().sum::<u8>() as f64 / moods.len() as f64;

        let min_mood = Mood::from_level(min_level).expect("levels always 1-5 from mood_stats");
        let max_mood = Mood::from_level(max_level).expect("levels always 1-5 from mood_stats");

        Some((min_mood, max_mood, avg))
    }
}

#[derive(Tabled)]
pub struct MoodRow {
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "MOOD")]
    mood: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "CONTENT")]
    content: String,
}

impl MoodRow {
    pub fn from_record(record: &MoodRecord) -> Self {
        Self {
            date: record.date.format("%Y-%m-%d").to_string(),
            mood: format!("{} {}", record.mood, record.mood.label()),
            tags: if record.tags.is_empty() {
                "-".to_string()
            } else {
                record.tags.join(", ")
            },
            content: if record.content.is_empty() {
                "-".to_string()
            } else {
                record.content.join("; ")
            },
        }
    }
}
