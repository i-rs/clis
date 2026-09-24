use i_rs_claw_core::app::Message;
use i_rs_claw_core::storage::ClawStorage;
use std::sync::Arc;

fn make_test_msgs(n: usize) -> Vec<Message> {
    (0..n)
        .map(|i| Message::User {
            text: format!("test message {}", i),
        })
        .collect()
}

pub async fn run(storage: &Arc<ClawStorage>) -> anyhow::Result<()> {
    let sid = "msg-test-session";

    // Create session entry so search can find it (search reads index.json)
    use i_rs_claw_core::session::{SessionMeta, SessionState};
    let now = chrono::Utc::now().timestamp();
    let meta = SessionMeta {
        id: sid.into(),
        title: "Test".into(),
        agent_id: "default".into(),
        user_id: "default".into(),
        state: SessionState::Active,
        created_at: now,
        updated_at: now,
        message_count: 0,
    };
    storage.sessions.upsert(&meta).await?;

    let msgs = make_test_msgs(3);
    storage.message_log.append_batch(sid, &msgs).await?;

    let loaded = storage.message_log.load(sid, 100).await?;
    assert_eq!(loaded.len(), 3, "expected 3 messages");

    let limited = storage.message_log.load(sid, 2).await?;
    assert_eq!(limited.len(), 2, "expected 2 messages with limit");

    let more = make_test_msgs(2);
    storage.message_log.append_batch(sid, &more).await?;
    let all = storage.message_log.load(sid, 100).await?;
    assert_eq!(all.len(), 5, "expected 5 messages after append");

    let found = storage.message_log.search("TEST MESSAGE", 10).await?;
    assert!(!found.is_empty(), "search should find results");

    let count = storage.message_log.count(sid).await?;
    assert_eq!(count, 5, "count should be 5");

    storage.message_log.delete_session(sid).await?;
    let after = storage.message_log.load(sid, 100).await?;
    assert!(after.is_empty(), "messages should be empty after delete");

    Ok(())
}
