use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    pub expiry_date: DateTime<Utc>,
    #[serde(default)]
    pub registrar: Option<String>,
    #[serde(skip)]
    pub password: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Domain {
    pub fn days_until_expiry(&self) -> i64 {
        let now = Utc::now();
        (self.expiry_date - now).num_days()
    }

    pub fn is_expired(&self) -> bool {
        self.days_until_expiry() < 0
    }

    pub fn is_expiring_soon(&self, days: i64) -> bool {
        let days_left = self.days_until_expiry();
        days_left >= 0 && days_left <= days
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct DomainStore {
    pub domains: std::collections::BTreeMap<String, Domain>,
}




impl DomainStore {
    pub fn add_entry(&mut self, entry: Domain) {
        self.domains.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, key: &str) -> Option<Domain> {
        self.domains.remove(key)
    }

    pub fn get_entry(&self, key: &str) -> Option<&Domain> {
        self.domains.get(key)
    }

    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut Domain> {
        self.domains.get_mut(key)
    }
}

#[derive(Tabled)]
pub struct DomainRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "EXPIRES")]
    expiry_date: String,
    #[tabled(rename = "DAYS_LEFT")]
    days_left: String,
    #[tabled(rename = "REGISTRAR")]
    registrar: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "REMARK")]
    remark: String,
}

impl DomainRow {
    pub fn from_domain(domain: &Domain) -> Self {
        let days = domain.days_until_expiry();
        let days_str = if domain.is_expired() {
            format!("{days} (expired)")
        } else if days <= 30 {
            format!("{days} (soon!)")
        } else {
            days.to_string()
        };

        Self {
            name: domain.name.clone(),
            expiry_date: domain.expiry_date.format("%Y-%m-%d").to_string(),
            days_left: days_str,
            registrar: domain.registrar.clone().unwrap_or_else(|| "-".to_string()),
            tags: if domain.tags.is_empty() {
                "-".to_string()
            } else {
                domain.tags.join(", ")
            },
            remark: if domain.remark.is_empty() {
                "-".to_string()
            } else {
                domain.remark.join(", ")
            },
        }
    }
}
