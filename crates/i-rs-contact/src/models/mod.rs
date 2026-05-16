use chrono::{DateTime, Utc};
use i_rs_core::storage::HasTags;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub relationship: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(default)]
    pub last_contact: Option<DateTime<Utc>>,
    #[serde(default)]
    pub contact_count: u32,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl HasTags for Contact {
    fn tags(&self) -> &[String] {
        &self.tags
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContactStore {
    pub entries: BTreeMap<String, Contact>,
}

#[allow(dead_code)]
impl ContactStore {
    pub fn add_entry(&mut self, entry: Contact) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<Contact> {
        self.entries.remove(name)
    }

    pub fn get_entry(&self, name: &str) -> Option<&Contact> {
        self.entries.get(name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut Contact> {
        self.entries.get_mut(name)
    }
}

#[derive(Tabled)]
pub struct ContactRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "PHONE")]
    phone: String,
    #[tabled(rename = "EMAIL")]
    email: String,
    #[tabled(rename = "RELATIONSHIP")]
    relationship: String,
    #[tabled(rename = "LAST CONTACT")]
    last_contact: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl ContactRow {
    pub fn from_contact(contact: &Contact) -> Self {
        Self {
            name: contact.name.clone(),
            phone: if contact.phone.is_empty() {
                "-".to_string()
            } else {
                contact.phone.clone()
            },
            email: if contact.email.is_empty() {
                "-".to_string()
            } else {
                contact.email.clone()
            },
            relationship: if contact.relationship.is_empty() {
                "-".to_string()
            } else {
                contact.relationship.clone()
            },
            last_contact: contact
                .last_contact
                .map_or_else(|| "-".to_string(), |dt| dt.format("%Y-%m-%d").to_string()),
            tags: if contact.tags.is_empty() {
                "-".to_string()
            } else {
                contact.tags.join(", ")
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub name: String,
    pub phone: String,
    pub email: String,
    pub relationship: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub last_contact: Option<String>,
    pub contact_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&Contact> for ListItem {
    fn from(contact: &Contact) -> Self {
        Self {
            name: contact.name.clone(),
            phone: contact.phone.clone(),
            email: contact.email.clone(),
            relationship: contact.relationship.clone(),
            tags: contact.tags.clone(),
            remark: contact.remark.clone(),
            last_contact: contact
                .last_contact
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            contact_count: contact.contact_count,
            created_at: contact.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: contact.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Stats {
    pub total_contacts: usize,
    pub by_relationship: std::collections::BTreeMap<String, usize>,
    pub by_tag: std::collections::BTreeMap<String, usize>,
    pub recent_contacts: Vec<ContactFrequency>,
    pub needs_reminder: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ContactFrequency {
    pub name: String,
    pub days_since_contact: i64,
    pub contact_count: u32,
}

impl Contact {
    pub fn days_since_last_contact(&self) -> Option<i64> {
        self.last_contact.map(|last| (Utc::now() - last).num_days())
    }
}
