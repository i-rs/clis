use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "high" | "h" | "3" => Some(Priority::High),
            "medium" | "med" | "m" | "2" => Some(Priority::Medium),
            "low" | "l" | "1" => Some(Priority::Low),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub content: Vec<String>,
    #[serde(default)]
    pub is_done: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Todo {
    pub fn toggle_done(&mut self) {
        self.is_done = !self.is_done;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoStore {
    pub todos: BTreeMap<String, Todo>,
}

impl Default for TodoStore {
    fn default() -> Self {
        Self {
            todos: BTreeMap::new(),
        }
    }
}

impl TodoStore {
    pub fn add_todo(&mut self, todo: Todo) {
        self.todos.insert(todo.name.clone(), todo);
    }

    pub fn remove_todo(&mut self, name: &str) -> Option<Todo> {
        self.todos.remove(name)
    }

    pub fn get_todo(&self, name: &str) -> Option<&Todo> {
        self.todos.get(name)
    }

    pub fn get_todo_mut(&mut self, name: &str) -> Option<&mut Todo> {
        self.todos.get_mut(name)
    }

    pub fn get_all_todos(&self) -> Vec<&Todo> {
        self.todos.values().collect()
    }

    pub fn get_pending_todos(&self) -> Vec<&Todo> {
        self.todos.values().filter(|t| !t.is_done).collect()
    }

    pub fn get_done_todos(&self) -> Vec<&Todo> {
        self.todos.values().filter(|t| t.is_done).collect()
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&Todo> {
        self.todos
            .values()
            .filter(|t| t.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn pending_count(&self) -> usize {
        self.todos.values().filter(|t| !t.is_done).count()
    }

    pub fn done_count(&self) -> usize {
        self.todos.values().filter(|t| t.is_done).count()
    }
}

#[derive(Tabled)]
pub struct TodoRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "TITLE")]
    title: String,
    #[tabled(rename = "PRIORITY")]
    priority: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl TodoRow {
    pub fn from_todo(todo: &Todo) -> Self {
        let status = if todo.is_done {
            format!("✓ {}", "Done")
        } else {
            "○ Pending".to_string()
        };

        let priority_str = match todo.priority {
            Priority::High => format!("🔴 {}", todo.priority.label()),
            Priority::Medium => format!("🟡 {}", todo.priority.label()),
            Priority::Low => format!("🟢 {}", todo.priority.label()),
        };

        Self {
            name: todo.name.clone(),
            title: todo.title.clone().unwrap_or_else(|| "-".to_string()),
            priority: priority_str,
            status,
            tags: if todo.tags.is_empty() {
                "-".to_string()
            } else {
                todo.tags.join(", ")
            },
        }
    }
}
