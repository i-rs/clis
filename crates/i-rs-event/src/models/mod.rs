use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Meeting,
    Gathering,
    Course,
    #[default]
    Other,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Meeting => write!(f, "meeting"),
            Self::Gathering => write!(f, "gathering"),
            Self::Course => write!(f, "course"),
            Self::Other => write!(f, "other"),
        }
    }
}

impl std::str::FromStr for EventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "meeting" => Ok(Self::Meeting),
            "gathering" => Ok(Self::Gathering),
            "course" => Ok(Self::Course),
            "other" => Ok(Self::Other),
            _ => Err(format!("Invalid event type: {s}")),
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

#[allow(dead_code)]
impl EventStore {
    pub fn add_entry(&mut self, entry: Event) {
        self.events.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, key: &str) -> Option<Event> {
        self.events.remove(key)
    }

    pub fn get_entry(&self, key: &str) -> Option<&Event> {
        self.events.get(key)
    }

    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut Event> {
        self.events.get_mut(key)
    }

    pub const fn new() -> Self {
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
