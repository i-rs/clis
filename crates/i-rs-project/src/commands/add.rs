use crate::models::{Priority, Project, ProjectStatus};
use crate::presentation::{print_error, print_success};
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
        print_error(&format!("Project '{}' already exists", name));
        anyhow::bail!("Project '{}' already exists", name);
    }

    let status = match status.as_deref() {
        Some("active") => ProjectStatus::Active,
        Some("onhold") | Some("on_hold") => ProjectStatus::OnHold,
        Some("completed") => ProjectStatus::Completed,
        Some("cancelled") => ProjectStatus::Cancelled,
        None => ProjectStatus::Active,
        _ => {
            print_error("Invalid status. Use: active, onhold, completed, or cancelled");
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
            print_error("Invalid priority. Use: low, medium, high, or urgent");
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
