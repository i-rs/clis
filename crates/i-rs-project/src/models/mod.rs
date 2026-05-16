use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

pub mod opt_ts_seconds {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt {
            Some(d) => serializer.serialize_some(&d.timestamp()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<i64> = Option::deserialize(deserializer)?;
        Ok(opt.and_then(|ts| DateTime::from_timestamp(ts, 0)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, with = "opt_ts_seconds")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: ProjectStatus,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(default)]
    pub milestones: Vec<Milestone>,
    #[serde(default)]
    pub tasks: Vec<Task>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ProjectStatus {
    #[default]
    Active,
    OnHold,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectStore {
    #[serde(default)]
    pub projects: BTreeMap<String, Project>,
}

impl ProjectStore {
    pub fn add_entry(&mut self, entry: Project) {
        self.projects.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<Project> {
        self.projects.remove(name)
    }

    pub fn get_entry(&self, name: &str) -> Option<&Project> {
        self.projects.get(name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut Project> {
        self.projects.get_mut(name)
    }
}

#[derive(Debug, Clone, Tabled)]
pub struct ProjectRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Priority")]
    pub priority: String,
    #[tabled(rename = "Milestones")]
    pub milestones: String,
    #[tabled(rename = "Tasks")]
    pub tasks: String,
    #[tabled(rename = "Progress")]
    pub progress: String,
}

impl ProjectRow {
    pub fn from_project(project: &Project) -> Self {
        let completed_milestones = project.milestones.iter().filter(|m| m.completed).count();
        let total_milestones = project.milestones.len();
        let completed_tasks = project.tasks.iter().filter(|t| t.completed).count();
        let total_tasks = project.tasks.len();

        let progress = if total_tasks > 0 {
            format!("{completed_tasks}/{total_tasks}")
        } else if total_milestones > 0 {
            format!("{completed_milestones}/{total_milestones} (m)")
        } else {
            "0%".to_string()
        };

        Self {
            name: project.name.clone(),
            status: format!("{:?}", project.status).to_lowercase(),
            priority: format!("{:?}", project.priority).to_lowercase(),
            milestones: format!("{completed_milestones}/{total_milestones}"),
            tasks: format!("{completed_tasks}/{total_tasks}"),
            progress,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectListItem {
    pub name: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl From<&Project> for ProjectListItem {
    fn from(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            description: project.description.clone(),
            status: format!("{:?}", project.status).to_lowercase(),
            priority: format!("{:?}", project.priority).to_lowercase(),
            tags: project.tags.clone(),
            created_at: project.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetail {
    pub name: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub milestones: Vec<Milestone>,
    pub tasks: Vec<Task>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Project> for ProjectDetail {
    fn from(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            description: project.description.clone(),
            status: format!("{:?}", project.status).to_lowercase(),
            priority: format!("{:?}", project.priority).to_lowercase(),
            tags: project.tags.clone(),
            remark: project.remark.clone(),
            milestones: project.milestones.clone(),
            tasks: project.tasks.clone(),
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}
