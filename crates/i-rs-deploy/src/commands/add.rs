use crate::models::{DeployRecord, DeployStatus};
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use uuid::Uuid;

pub fn handle_add(
    project: String,
    environment: String,
    version: String,
    status: DeployStatus,
    rollback_from: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let entry = DeployRecord {
        id: id.clone(),
        project: project.clone(),
        environment: environment.clone(),
        version: version.clone(),
        status,
        deployed_at: now,
        rollback_from,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Deploy record created: {} ({}/{}/{})",
        id.green(),
        project.green(),
        environment.green(),
        version.green()
    ));

    Ok(())
}
