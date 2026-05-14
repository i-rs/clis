use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PodcastStatus {
    NotStarted,
    InProgress,
    Completed,
}

impl Default for PodcastStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

impl std::fmt::Display for PodcastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => write!(f, "not_started"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
        }
    }
}

impl From<&str> for PodcastStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "in_progress" | "inprogress" | "progress" | "ongoing" => Self::InProgress,
            "completed" | "done" | "finished" => Self::Completed,
            _ => Self::NotStarted,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Podcast {
    pub name: String,
    pub author: Option<String>,
    pub duration_secs: Option<i64>,
    pub current_position_secs: Option<i64>,
    pub status: PodcastStatus,
    pub notes: Vec<String>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastStore {
    pub podcasts: BTreeMap<String, Podcast>,
}

impl Default for PodcastStore {
    fn default() -> Self {
        Self {
            podcasts: BTreeMap::new(),
        }
    }
}

impl PodcastStore {
    pub fn add_podcast(&mut self, podcast: Podcast) {
        self.podcasts.insert(podcast.name.clone(), podcast);
    }

    pub fn remove_podcast(&mut self, name: &str) -> Option<Podcast> {
        self.podcasts.remove(name)
    }

    pub fn get_podcast(&self, name: &str) -> Option<&Podcast> {
        self.podcasts.get(name)
    }

    pub fn get_mut_podcast(&mut self, name: &str) -> Option<&mut Podcast> {
        self.podcasts.get_mut(name)
    }

    pub fn get_all_podcasts(&self) -> Vec<&Podcast> {
        self.podcasts.values().collect()
    }

    pub fn get_by_status(&self, status: PodcastStatus) -> Vec<&Podcast> {
        self.podcasts
            .values()
            .filter(|p| p.status == status)
            .collect()
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&Podcast> {
        self.podcasts
            .values()
            .filter(|p| p.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn podcast_stats(&self) -> PodcastStats {
        let total = self.podcasts.len();
        let not_started = self
            .podcasts
            .values()
            .filter(|p| p.status == PodcastStatus::NotStarted)
            .count();
        let in_progress = self
            .podcasts
            .values()
            .filter(|p| p.status == PodcastStatus::InProgress)
            .count();
        let completed = self
            .podcasts
            .values()
            .filter(|p| p.status == PodcastStatus::Completed)
            .count();

        let total_duration: i64 = self
            .podcasts
            .values()
            .filter_map(|p| p.duration_secs)
            .sum();
        let total_listened: i64 = self
            .podcasts
            .values()
            .filter_map(|p| p.current_position_secs)
            .sum();

        PodcastStats {
            total,
            not_started,
            in_progress,
            completed,
            total_duration_secs: total_duration,
            total_listened_secs: total_listened,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastStats {
    pub total: usize,
    pub not_started: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub total_duration_secs: i64,
    pub total_listened_secs: i64,
}

impl PodcastStats {
    pub fn total_progress_percent(&self) -> f64 {
        if self.total_duration_secs == 0 {
            0.0
        } else {
            (self.total_listened_secs as f64 / self.total_duration_secs as f64) * 100.0
        }
    }
}

#[derive(Tabled)]
pub struct PodcastRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "AUTHOR")]
    author: String,
    #[tabled(rename = "DURATION")]
    duration: String,
    #[tabled(rename = "POSITION")]
    position: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl PodcastRow {
    pub fn from_podcast(podcast: &Podcast) -> Self {
        Self {
            name: podcast.name.clone(),
            author: podcast.author.clone().unwrap_or_else(|| "-".to_string()),
            duration: format_duration(podcast.duration_secs),
            position: format_duration(podcast.current_position_secs),
            status: format_status(&podcast.status),
            tags: if podcast.tags.is_empty() {
                "-".to_string()
            } else {
                podcast.tags.join(", ")
            },
        }
    }
}

fn format_duration(secs: Option<i64>) -> String {
    match secs {
        Some(s) => {
            let hours = s / 3600;
            let minutes = (s % 3600) / 60;
            let seconds = s % 60;
            if hours > 0 {
                format!("{}:{:02}:{:02}", hours, minutes, seconds)
            } else {
                format!("{}:{:02}", minutes, seconds)
            }
        }
        None => "-".to_string(),
    }
}

fn format_status(status: &PodcastStatus) -> String {
    match status {
        PodcastStatus::NotStarted => "○".to_string(),
        PodcastStatus::InProgress => "◐".to_string(),
        PodcastStatus::Completed => "●".to_string(),
    }
}

pub trait HasTags {
    fn get_tags(&self) -> &[String];
    fn has_tag(&self, tag: &str) -> bool {
        self.get_tags().iter().any(|t| t == tag)
    }
}

impl HasTags for Podcast {
    fn get_tags(&self) -> &[String] {
        &self.tags
    }
}
