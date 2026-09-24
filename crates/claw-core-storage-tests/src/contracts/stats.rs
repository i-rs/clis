use chrono::Utc;
use i_rs_claw_core::stats::TokenRecord;
use i_rs_claw_core::storage::ClawStorage;
use std::sync::Arc;

pub async fn run(storage: &Arc<ClawStorage>) -> anyhow::Result<()> {
    let now = Utc::now().timestamp();

    let records: Vec<TokenRecord> = (0..5)
        .map(|i| TokenRecord {
            id: format!("stat-{}", i),
            timestamp: now - i as i64 * 86400,
            user_id: "default".into(),
            agent_id: "default".into(),
            model: "gpt-4".into(),
            provider: "openai".into(),
            prompt_tokens: 100 + i * 10,
            completion_tokens: 50 + i * 5,
            total_tokens: 150 + i * 15,
            has_tool_calls: i % 2 == 0,
            tool_call_count: if i % 2 == 0 { 2 } else { 0 },
            react_rounds: 1,
            success: true,
            latency_ms: 500 + i as u64 * 100,
            estimated_cost_usd: 0.001 * (i as f64 + 1.0),
            trace_id: format!("trace-{}", i),
        })
        .collect();
    storage.stats.upsert_batch(&records).await?;

    let all = storage
        .stats
        .read_range(Some(now - 86400 * 30), Some(now + 86400))
        .await?;
    assert_eq!(all.len(), 5, "should have 5 records in range");

    let recent = storage
        .stats
        .read_range(Some(now - 86400 * 2), Some(now + 86400))
        .await?;
    assert!(!recent.is_empty(), "should have >=1 recent record");

    let pruned = storage.stats.prune(3).await?;
    assert!(pruned > 0, "prune should remove some records");

    let after_prune = storage
        .stats
        .read_range(Some(now - 86400 * 365), Some(now + 86400))
        .await?;
    assert!(
        after_prune.len() < 5,
        "prune should remove old records, got {}",
        after_prune.len()
    );

    Ok(())
}
