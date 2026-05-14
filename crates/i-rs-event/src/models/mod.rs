use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Meeting,
    Gathering,
    Course,
    Other,
}

impl Default for EventType {
    fn default() -> Self {
        EventType::Other
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Meeting => write!(f, "meeting"),
            EventType::Gathering => write!(f, "gathering"),
            EventType::Course => write!(f, "course"),
            EventType::Other => write!(f, "other"),
        }
    }
}

impl std::str::FromStr for EventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "meeting" => Ok(EventType::Meeting),
            "gathering" => Ok(EventType::Gathering),
            "course" => Ok(EventType::Course),
            "other" => Ok(EventType::Other),
            _ => Err(format!("Invalid event type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub date: DateTime<Utc>,
    #[serde(default)]
    pub event_type: EventType,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub participants: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Event {
    pub fn new(name: String, date: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            name,
            date,
            event_type: EventType::Other,
            location: String::new(),
            participants: Vec::new(),
            tags: Vec::new(),
            remark: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Tabled, Debug)]
#[tabled(rename_all = "PascalCase")]
pub struct EventRow {
    pub name: String,
    pub date: String,
    #[tabled(rename = "type")]
    pub event_type: String,
    pub location: String,
    pub participants: String,
    pub tags: String,
}

impl EventRow {
    pub fn from_event(event: &Event) -> Self {
        Self {
            name: event.name.clone(),
            date: event.date.format("%Y-%m-%d %H:%M").to_string(),
            event_type: event.event_type.to_string(),
            location: event.location.clone(),
            participants: event.participants.join(", "),
            tags: event.tags.join(", "),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStore {
    pub events: std::collections::BTreeMap<String, Event>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            events: std::collections::BTreeMap::new(),
        }
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}
