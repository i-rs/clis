use crate::models::{DeployRecord, DeployStatus};
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use uuid::Uuid;

pub fn handle_rollback(
    project: String,
    environment: String,
    rollback_to_id: Option<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let mut candidates: Vec<&DeployRecord> = store
        .entries
        .values()
        .filter(|r| r.project == project && r.environment == environment)
        .filter(|r| matches!(r.status, DeployStatus::Success))
        .collect();

    candidates.sort_by_key(|e| std::cmp::Reverse(e.deployed_at));

    let target_record = if let Some(ref id) = rollback_to_id {
        candidates
            .iter()
            .find(|r| r.id == *id || r.id.starts_with(id))
            .copied()
    } else {
        candidates.get(1).copied()
    };

    if let Some(target) = target_record {
        let rollback_from = target.id.clone();
        let target_version = target.version.clone();
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let entry = DeployRecord {
            id: id.clone(),
            project: target.project.clone(),
            environment: target.environment.clone(),
            version: target.version.clone(),
            status: DeployStatus::RolledBack,
            deployed_at: now,
            rollback_from: Some(rollback_from),
            tags: vec!["rollback".to_string()],
            remark: vec![],
            created_at: now,
            updated_at: now,
        };

        store.add_entry(entry);
        storage::save_store(&store)?;

        print_success(&format!(
            "✓ Rolled back to {} ({})",
            target_version.green(),
            &id[..8.min(id.len())]
        ));
    } else {
        if rollback_to_id.is_some() {
            print_error("Target deployment not found");
        } 
        anyhow::bail!("Rollback failed");
    }

    Ok(())
}
