use i_rs_claw_core::session::{SessionMeta, SessionState};
use i_rs_claw_core::storage::ClawStorage;
use std::sync::Arc;

pub async fn run(storage: &Arc<ClawStorage>) -> anyhow::Result<()> {
    let now = chrono::Utc::now().timestamp();

    let meta = SessionMeta {
        id: "test-session-1".into(),
        title: "Test".into(),
        agent_id: "default".into(),
        user_id: "default".into(),
        state: SessionState::Active,
        created_at: now,
        updated_at: now,
        message_count: 0,
    };
    storage.sessions.upsert(&meta).await?;

    let all = storage.sessions.load_all().await?;
    assert!(!all.is_empty(), "expected >= 1 sessions, got {}", all.len());

    let loaded = storage.sessions.get_one("test-session-1").await?;
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().title, "Test");

    let updated = SessionMeta {
        message_count: 5,
        state: SessionState::Completed,
        updated_at: chrono::Utc::now().timestamp(),
        ..meta
    };
    storage.sessions.upsert(&updated).await?;
    let reloaded = storage.sessions.get_one("test-session-1").await?.unwrap();
    assert_eq!(reloaded.message_count, 5);
    assert!(matches!(reloaded.state, SessionState::Completed));

    let count = storage.sessions.count().await?;
    assert!(count >= 1, "count should be >= 1");

    storage.sessions.delete_one("test-session-1").await?;
    let after_delete = storage.sessions.get_one("test-session-1").await?;
    assert!(after_delete.is_none());

    Ok(())
}
