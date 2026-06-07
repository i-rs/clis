use std::sync::Arc;
use std::collections::HashMap;
use i_rs_claw_core::storage::ClawStorage;

pub async fn run(storage: &Arc<ClawStorage>) -> anyhow::Result<()> {
    let agent = "tool-cache-agent";

    let mut cache = HashMap::new();
    cache.insert("weight_tool".into(), "Weight tracking tool docs".into());
    cache.insert("todo_tool".into(), "Todo management".into());
    storage.tool_cache.save(agent, &cache).await?;

    let loaded = storage.tool_cache.load(agent).await?;
    assert_eq!(loaded.len(), 2, "expected 2 tools");
    assert_eq!(
        loaded.get("weight_tool").unwrap(),
        "Weight tracking tool docs"
    );

    let other = storage.tool_cache.load("other-agent").await?;
    assert!(other.is_empty(), "other agent should have no cache");

    storage.tool_cache.save(agent, &HashMap::new()).await?;
    let after = storage.tool_cache.load(agent).await?;
    assert!(after.is_empty(), "cache should be empty after clearing");

    Ok(())
}
