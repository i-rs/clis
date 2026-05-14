use chrono::{DateTime, Utc};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remind {
    pub name: String,
    pub event_date: DateTime<Utc>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub content: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(default)]
    pub is_done: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Remind {
    pub fn days_until_event(&self) -> i64 {
        let now = Utc::now();
        (self.event_date - now).num_days()
    }

    pub fn is_past(&self) -> bool {
        self.days_until_event() < 0
    }

    pub fn is_today(&self) -> bool {
        let days = self.days_until_event();
        days >= 0 && days < 1
    }

    pub fn is_upcoming(&self, days: i64) -> bool {
        let days_left = self.days_until_event();
        days_left >= 0 && days_left <= days
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemindStore {
    pub reminds: std::collections::BTreeMap<String, Remind>,
}

impl Default for RemindStore {
    fn default() -> Self {
        Self {
            reminds: std::collections::BTreeMap::new(),
        }
    }
}

#[derive(Tabled)]
pub struct RemindRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "DATE")]
    event_date: String,
    #[tabled(rename = "DAYS_LEFT")]
    days_left: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "REMARK")]
    remark: String,
}

impl RemindRow {
    pub fn from_remind(remind: &Remind) -> Self {
        let days = remind.days_until_event();
        let (days_str, status) = if remind.is_done {
            (format!("-"), "DONE".green().to_string())
        } else if remind.is_past() {
            (format!("{} days ago", days.abs()), "PAST".dimmed().to_string())
        } else if remind.is_today() {
            ("TODAY!".red().bold().to_string(), "TODAY".red().bold().to_string())
        } else if days <= 7 {
            (format!("{} days", days), format!("{}", "SOON".yellow()))
        } else {
            (format!("{} days", days), "UPCOMING".cyan().to_string())
        };

        Self {
            name: remind.name.clone(),
            event_date: remind.event_date.format("%Y-%m-%d %H:%M").to_string(),
            days_left: days_str,
            status,
            tags: if remind.tags.is_empty() {
                "-".to_string()
            } else {
                remind.tags.join(", ")
            },
            remark: if remind.remark.is_empty() {
                "-".to_string()
            } else {
                remind.remark.join(", ")
            },
        }
    }
}
