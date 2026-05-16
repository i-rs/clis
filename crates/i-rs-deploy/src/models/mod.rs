use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
pub enum DeployStatus {
    #[value(name = "success")]
    Success,
    #[value(name = "failed")]
    Failed,
    #[value(name = "rolling_back")]
    RollingBack,
    #[value(name = "rolled_back")]
    RolledBack,
}

impl std::fmt::Display for DeployStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failed => write!(f, "failed"),
            Self::RollingBack => write!(f, "rolling_back"),
            Self::RolledBack => write!(f, "rolled_back"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployRecord {
    pub id: String,
    pub project: String,
    pub environment: String,
    pub version: String,
    pub status: DeployStatus,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub deployed_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback_from: Option<String>,
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
pub struct DeployStore {
    pub entries: BTreeMap<String, DeployRecord>,
}

impl DeployStore {
    pub fn add_entry(&mut self, entry: DeployRecord) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<DeployRecord> {
        self.entries.remove(id)
    }

    pub fn get_entry(&self, id: &str) -> Option<&DeployRecord> {
        self.entries.get(id)
    }

    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut DeployRecord> {
        self.entries.get_mut(id)
    }
}

#[derive(Tabled)]
pub struct DeployRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "PROJECT")]
    project: String,
    #[tabled(rename = "ENV")]
    environment: String,
    #[tabled(rename = "VERSION")]
    version: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "DEPLOYED_AT")]
    deployed_at: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl DeployRow {
    pub fn from_record(record: &DeployRecord) -> Self {
        Self {
            id: record.id[..8.min(record.id.len())].to_string(),
            project: record.project.clone(),
            environment: record.environment.clone(),
            version: record.version.clone(),
            status: record.status.to_string(),
            deployed_at: record.deployed_at.format("%Y-%m-%d %H:%M").to_string(),
            tags: if record.tags.is_empty() {
                "-".to_string()
            } else {
                record.tags.join(", ")
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub project: String,
    pub environment: String,
    pub version: String,
    pub status: String,
    pub deployed_at: String,
    pub rollback_from: Option<String>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&DeployRecord> for ListItem {
    fn from(record: &DeployRecord) -> Self {
        Self {
            id: record.id.clone(),
            project: record.project.clone(),
            environment: record.environment.clone(),
            version: record.version.clone(),
            status: record.status.to_string(),
            deployed_at: record.deployed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            rollback_from: record.rollback_from.clone(),
            tags: record.tags.clone(),
            remark: record.remark.clone(),
            created_at: record.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
