use crate::models::{Priority, ProjectStatus};
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let project = match store.get_entry_mut(&name) {
        Some(p) => p,
        None => {
            anyhow::bail!("Project '{name}' not found");
        }
    };

    let mut updated = false;

    if let Some(desc) = description {
        project.description = desc;
        updated = true;
    }

    if let Some(s) = status {
        let new_status = match s.as_str() {
            "active" => ProjectStatus::Active,
            "onhold" | "on_hold" => ProjectStatus::OnHold,
            "completed" => ProjectStatus::Completed,
            "cancelled" => ProjectStatus::Cancelled,
            _ => {
                anyhow::bail!("Invalid status");
            }
        };
        project.status = new_status;
        updated = true;
    }

    if let Some(p) = priority {
        let new_priority = match p.as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            "urgent" => Priority::Urgent,
            _ => {
                anyhow::bail!("Invalid priority");
            }
        };
        project.priority = new_priority;
        updated = true;
    }

    if let Some(tags) = tag {
        project.tags = tags;
        updated = true;
    }

    if let Some(remarks) = remark {
        project.remark = remarks;
        updated = true;
    }

    if updated {
        project.updated_at = Utc::now();
        storage::save_store(&store)?;
        print_success(&format!("✓ Project '{}' updated successfully", name.green()));
    } else {
        print_success("No changes made");
    }

    Ok(())
}
