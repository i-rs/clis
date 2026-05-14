use crate::models::{Priority, Project, ProjectStatus};
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if storage::project_exists(&store, &name) {
        anyhow::bail!("Project '{}' already exists", name);
    }

    let status = match status.as_deref() {
        Some("active") => ProjectStatus::Active,
        Some("onhold") | Some("on_hold") => ProjectStatus::OnHold,
        Some("completed") => ProjectStatus::Completed,
        Some("cancelled") => ProjectStatus::Cancelled,
        None => ProjectStatus::Active,
        _ => {
            anyhow::bail!("Invalid status");
        }
    };

    let priority = match priority.as_deref() {
        Some("low") => Priority::Low,
        Some("medium") => Priority::Medium,
        Some("high") => Priority::High,
        Some("urgent") => Priority::Urgent,
        None => Priority::Medium,
        _ => {
            anyhow::bail!("Invalid priority");
        }
    };

    let now = Utc::now();
    let project = Project {
        name: name.clone(),
        description: description.unwrap_or_default(),
        status,
        priority,
        tags: tag,
        remark,
        milestones: Vec::new(),
        tasks: Vec::new(),
        created_at: now,
        updated_at: now,
    };

    storage::add_project(&mut store, project);
    storage::save_store(&store)?;

    print_success(&format!("✓ Project '{}' added successfully", name.green()));

    Ok(())
}
