use std::sync::Arc;
use i_rs_claw_core::storage::ClawStorage;

/// Run all 7 contracts against a storage backend.
/// Returns (contract_name, Result) for each.
pub async fn run_all_contracts(
    storage: Arc<ClawStorage>,
) -> Vec<(&'static str, anyhow::Result<()>)> {
    let mut results = Vec::new();
    results.push(("session", crate::contracts::session::run(&storage).await));
    results.push(("message_log", crate::contracts::message_log::run(&storage).await));
    results.push(("memory", crate::contracts::memory::run(&storage).await));
    results.push(("stats", crate::contracts::stats::run(&storage).await));
    results.push(("tool_cache", crate::contracts::tool_cache::run(&storage).await));
    results.push(("skill", crate::contracts::skill::run(&storage).await));
    results
}

pub mod helpers;
