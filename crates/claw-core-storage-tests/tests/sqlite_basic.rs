use i_rs_claw_core::app::Message;
use i_rs_claw_core::session::{SessionMeta, SessionState};
use i_rs_claw_core::stats::TokenRecord;
use i_rs_claw_core::storage::config_store::{
    AgentConfigRow, ConfigStore, DashboardUserRow, McpServerConfigRow, ProviderConfigRow,
};
use i_rs_claw_core::storage::sql::sqlite::SqliteBackend;
use i_rs_claw_core::storage::ClawStorage;
use std::path::PathBuf;

fn temp_db(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("claw-storage-test-{}-{}.db", label, uuid::Uuid::new_v4()))
}

async fn create_config_store(label: &str) -> (ConfigStore, PathBuf) {
    let path = temp_db(label);
    let backend = SqliteBackend::new(path.clone())
        .await
        .expect("open SQLite for config store");
    (backend.into_config_store(), path)
}

async fn create_storage(label: &str) -> (ClawStorage, PathBuf) {
    let path = temp_db(label);
    let storage = ClawStorage::sqlite(path.clone())
        .await
        .expect("open SQLite for storage");
    (storage, path)
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

// ── Test 1: Agent config CRUD ──

#[tokio::test]
async fn test_agent_config_crud() {
    let (cfg, path) = create_config_store("agent-crud").await;

    let row = AgentConfigRow {
        user_id: "user-1".into(),
        agent_id: "analyst".into(),
        provider_ref: Some("default".into()),
        provider: "openai".into(),
        api_key: "sk-test".into(),
        base_url: "https://api.deepseek.com".into(),
        model: "deepseek-chat".into(),
        enabled_tools: vec!["weight".into(), "mood".into()],
        system_prompt: "You are a data analyst.".into(),
        system_prompt_file: None,
        capabilities: vec!["数据分析".into()],
        execution_mode: "React".into(),
        created_at: now(),
        updated_at: now(),
    };

    cfg.agent_configs
        .upsert(&row)
        .await
        .expect("upsert agent config");

    let all = cfg
        .agent_configs
        .load_all("user-1")
        .await
        .expect("load_all agent configs");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].agent_id, "analyst");
    assert_eq!(all[0].model, "deepseek-chat");
    assert_eq!(all[0].enabled_tools.len(), 2);

    // Update the same agent
    let mut updated = row.clone();
    updated.model = "deepseek-v3".into();
    cfg.agent_configs
        .upsert(&updated)
        .await
        .expect("upsert updated agent config");

    let all = cfg.agent_configs.load_all("user-1").await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].model, "deepseek-v3");

    // Another user should see nothing
    let other = cfg.agent_configs.load_all("user-2").await.unwrap();
    assert!(other.is_empty());

    // Delete
    cfg.agent_configs
        .delete("user-1", "analyst")
        .await
        .expect("delete agent config");
    assert!(cfg.agent_configs.load_all("user-1").await.unwrap().is_empty());

    let _ = std::fs::remove_file(&path);
}

// ── Test 2: Provider config CRUD ──

#[tokio::test]
async fn test_provider_config_crud() {
    let (cfg, path) = create_config_store("provider-crud").await;

    let row = ProviderConfigRow {
        name: "deepseek".into(),
        provider: "openai".into(),
        api_key: "sk-deepseek".into(),
        base_url: "https://api.deepseek.com".into(),
        model: "deepseek-chat".into(),
        created_at: now(),
        updated_at: now(),
    };

    cfg.provider_configs
        .upsert(&row)
        .await
        .expect("upsert provider config");

    let all = cfg
        .provider_configs
        .load_all()
        .await
        .expect("load_all providers");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name, "deepseek");
    assert_eq!(all[0].provider, "openai");

    // Add a second provider
    let row2 = ProviderConfigRow {
        name: "ollama".into(),
        provider: "ollama".into(),
        api_key: String::new(),
        base_url: "http://localhost:11434".into(),
        model: "llama3".into(),
        created_at: now(),
        updated_at: now(),
    };
    cfg.provider_configs.upsert(&row2).await.unwrap();

    let all = cfg.provider_configs.load_all().await.unwrap();
    assert_eq!(all.len(), 2);

    // Delete first
    cfg.provider_configs
        .delete("deepseek")
        .await
        .expect("delete provider");
    let all = cfg.provider_configs.load_all().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name, "ollama");

    let _ = std::fs::remove_file(&path);
}

// ── Test 3: Dashboard users CRUD ──

#[tokio::test]
async fn test_dashboard_users_crud() {
    let (cfg, path) = create_config_store("dashboard-crud").await;

    let user = DashboardUserRow {
        user_id: "alice".into(),
        token_hash: "hash-abc123".into(),
        display_name: "Alice".into(),
        created_at: now(),
        updated_at: now(),
    };

    cfg.dashboard_users
        .upsert(&user)
        .await
        .expect("upsert dashboard user");

    let all = cfg
        .dashboard_users
        .load_all()
        .await
        .expect("load_all dashboard users");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].user_id, "alice");
    assert_eq!(all[0].display_name, "Alice");

    // find_by_token_hash
    let found = cfg
        .dashboard_users
        .find_by_token_hash("hash-abc123")
        .await
        .expect("find_by_token_hash");
    assert!(found.is_some());
    assert_eq!(found.unwrap().user_id, "alice");

    // Not found
    let missing = cfg
        .dashboard_users
        .find_by_token_hash("nonexistent")
        .await
        .unwrap();
    assert!(missing.is_none());

    // Add second user
    let user2 = DashboardUserRow {
        user_id: "bob".into(),
        token_hash: "hash-xyz789".into(),
        display_name: "Bob".into(),
        created_at: now(),
        updated_at: now(),
    };
    cfg.dashboard_users.upsert(&user2).await.unwrap();
    assert_eq!(cfg.dashboard_users.load_all().await.unwrap().len(), 2);

    // Delete alice
    cfg.dashboard_users
        .delete("alice")
        .await
        .expect("delete dashboard user");
    let all = cfg.dashboard_users.load_all().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].user_id, "bob");

    let _ = std::fs::remove_file(&path);
}

// ── Test 4: MCP server config CRUD ──

#[tokio::test]
async fn test_mcp_server_crud() {
    let (cfg, path) = create_config_store("mcp-crud").await;

    let row = McpServerConfigRow {
        user_id: "user-1".into(),
        agent_id: Some("analyst".into()),
        name: "playwright".into(),
        transport_type: "stdio".into(),
        command: Some("npx".into()),
        args_json: Some(r#"["-y","@anthropic-ai/claude-code-mcp"]"#.into()),
        url: None,
        env_json: None,
        enabled: true,
    };

    cfg.mcp_servers
        .upsert(&row)
        .await
        .expect("upsert mcp server config");

    // Load for specific user + agent
    let servers = cfg
        .mcp_servers
        .load_for("user-1", Some("analyst"))
        .await
        .expect("load_for mcp servers");
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "playwright");
    assert_eq!(servers[0].transport_type, "stdio");
    assert!(servers[0].enabled);

    // Load for different agent should be empty
    let servers = cfg.mcp_servers.load_for("user-1", Some("other")).await.unwrap();
    assert!(servers.is_empty());

    // Load for different user should be empty
    let servers = cfg.mcp_servers.load_for("user-2", Some("analyst")).await.unwrap();
    assert!(servers.is_empty());

    // Add global (agent_id = None) entry
    let global = McpServerConfigRow {
        user_id: "user-1".into(),
        agent_id: None,
        name: "global-tool".into(),
        transport_type: "sse".into(),
        command: None,
        args_json: None,
        url: Some("http://localhost:8080/sse".into()),
        env_json: None,
        enabled: false,
    };
    cfg.mcp_servers.upsert(&global).await.unwrap();

    // Load global entries
    let servers = cfg.mcp_servers.load_for("user-1", None).await.unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "global-tool");

    // Delete agent-specific entry
    cfg.mcp_servers
        .delete("user-1", Some("analyst"), "playwright")
        .await
        .expect("delete mcp server");
    let servers = cfg
        .mcp_servers
        .load_for("user-1", Some("analyst"))
        .await
        .unwrap();
    assert!(servers.is_empty());

    // Global entry still exists
    let servers = cfg.mcp_servers.load_for("user-1", None).await.unwrap();
    assert_eq!(servers.len(), 1);

    let _ = std::fs::remove_file(&path);
}

// ── Test 5: App settings CRUD ──

#[tokio::test]
async fn test_app_settings_crud() {
    let (cfg, path) = create_config_store("settings-crud").await;

    // Set a string value
    let val = serde_json::json!("dark");
    cfg.app_settings
        .set("theme", &val)
        .await
        .expect("set app setting");

    let got = cfg
        .app_settings
        .get("theme")
        .await
        .expect("get app setting");
    assert_eq!(got, Some(serde_json::json!("dark")));

    // Set a numeric value
    cfg.app_settings
        .set("max_tokens", &serde_json::json!(4096))
        .await
        .unwrap();
    let got = cfg.app_settings.get("max_tokens").await.unwrap();
    assert_eq!(got, Some(serde_json::json!(4096)));

    // Set a complex value
    let complex = serde_json::json!({"enabled": true, "threshold": 0.75});
    cfg.app_settings
        .set("quality_config", &complex)
        .await
        .unwrap();

    // load_all
    let all = cfg.app_settings.load_all().await.expect("load_all settings");
    assert_eq!(all.len(), 3);

    // Verify specific keys exist
    let keys: Vec<&str> = all.iter().map(|r| r.key.as_str()).collect();
    assert!(keys.contains(&"theme"));
    assert!(keys.contains(&"max_tokens"));
    assert!(keys.contains(&"quality_config"));

    // Overwrite existing key
    cfg.app_settings
        .set("theme", &serde_json::json!("light"))
        .await
        .unwrap();
    let got = cfg.app_settings.get("theme").await.unwrap();
    assert_eq!(got, Some(serde_json::json!("light")));
    // load_all should still have 3 entries
    assert_eq!(cfg.app_settings.load_all().await.unwrap().len(), 3);

    // Get non-existent key
    let missing = cfg.app_settings.get("nonexistent").await.unwrap();
    assert!(missing.is_none());

    let _ = std::fs::remove_file(&path);
}

// ── Test 6: Message log append + search ──

#[tokio::test]
async fn test_message_log_append_and_search() {
    let (storage, path) = create_storage("msg-log").await;

    let session_id = uuid::Uuid::new_v4().to_string();
    let meta = SessionMeta {
        id: session_id.clone(),
        title: "Test Chat".into(),
        agent_id: "default".into(),
        user_id: "default".into(),
        state: SessionState::Active,
        created_at: now(),
        updated_at: now(),
        message_count: 0,
    };
    storage.sessions.upsert(&meta).await.expect("upsert session");

    let messages = vec![
        Message::User {
            text: "What is the weather in Tokyo?".into(),
        },
        Message::Assistant {
            text: "The weather in Tokyo is sunny, 22\u{00b0}C.".into(),
            reasoning: String::new(),
            token_usage: None,
        },
        Message::ToolCall {
            name: "web_search".into(),
            args: r#"{"query":"Tokyo weather"}"#.into(),
            result: r#"{"temp":22,"condition":"sunny"}"#.into(),
            step: 1,
            total_steps: 1,
        },
    ];

    storage
        .message_log
        .append_batch(&session_id, &messages)
        .await
        .expect("append_batch");

    // Load all
    let loaded = storage
        .message_log
        .load(&session_id, usize::MAX)
        .await
        .expect("load messages");
    assert_eq!(loaded.len(), 3, "should load 3 messages");

    match &loaded[0] {
        Message::User { text } => assert!(text.contains("weather")),
        other => panic!("expected User message, got {:?}", other),
    }
    match &loaded[2] {
        Message::ToolCall { name, .. } => assert_eq!(name, "web_search"),
        other => panic!("expected ToolCall, got {:?}", other),
    }

    // Count
    let cnt = storage
        .message_log
        .count(&session_id)
        .await
        .expect("count messages");
    assert_eq!(cnt, 3);

    // Search - case insensitive; both User and Assistant contain "Tokyo"
    let results = storage
        .message_log
        .search("tokyo", 10)
        .await
        .expect("search messages");
    assert_eq!(results.len(), 2, "User and Assistant both mention Tokyo");
    assert!(results[0]
        .excerpt
        .to_lowercase()
        .contains("weather in tokyo"));

    // Search for tool call
    let results = storage
        .message_log
        .search("web_search", 10)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].excerpt.contains("web_search"));

    // Search no match
    let results = storage
        .message_log
        .search("zzzz_nonexistent", 10)
        .await
        .unwrap();
    assert!(results.is_empty());

    // Delete session messages
    storage
        .message_log
        .delete_session(&session_id)
        .await
        .expect("delete_session");
    let loaded = storage.message_log.load(&session_id, 100).await.unwrap();
    assert!(loaded.is_empty());

    let _ = std::fs::remove_file(&path);
}

// ── Test 7: Session meta upsert + lookup ──

#[tokio::test]
async fn test_session_meta_upsert_and_lookup() {
    let (storage, path) = create_storage("session-meta").await;

    let id1 = uuid::Uuid::new_v4().to_string();
    let id2 = uuid::Uuid::new_v4().to_string();

    // Upsert first session
    let s1 = SessionMeta {
        id: id1.clone(),
        title: "First Chat".into(),
        agent_id: "analyst".into(),
        user_id: "user-1".into(),
        state: SessionState::Active,
        created_at: now(),
        updated_at: now(),
        message_count: 0,
    };
    storage.sessions.upsert(&s1).await.expect("upsert s1");

    // Upsert second session
    let s2 = SessionMeta {
        id: id2.clone(),
        title: "Second Chat".into(),
        agent_id: "default".into(),
        user_id: "user-1".into(),
        state: SessionState::Completed,
        created_at: now(),
        updated_at: now(),
        message_count: 5,
    };
    storage.sessions.upsert(&s2).await.expect("upsert s2");

    // Count
    let cnt = storage.sessions.count().await.expect("count sessions");
    assert_eq!(cnt, 2);

    // Get one
    let got = storage
        .sessions
        .get_one(&id1)
        .await
        .expect("get_one session");
    assert!(got.is_some());
    let got = got.unwrap();
    assert_eq!(got.title, "First Chat");
    assert_eq!(got.agent_id, "analyst");
    assert_eq!(got.message_count, 0);

    let got = storage.sessions.get_one(&id2).await.unwrap().unwrap();
    assert_eq!(got.state, SessionState::Completed);
    assert_eq!(got.message_count, 5);

    // Get non-existent
    let missing = storage
        .sessions
        .get_one("nonexistent")
        .await
        .unwrap();
    assert!(missing.is_none());

    // Upsert: update existing
    let mut s1_updated = s1.clone();
    s1_updated.title = "Updated Chat".into();
    s1_updated.message_count = 10;
    storage.sessions.upsert(&s1_updated).await.unwrap();
    let got = storage.sessions.get_one(&id1).await.unwrap().unwrap();
    assert_eq!(got.title, "Updated Chat");
    assert_eq!(got.message_count, 10);

    // Delete one
    storage
        .sessions
        .delete_one(&id1)
        .await
        .expect("delete_one session");
    assert_eq!(storage.sessions.count().await.unwrap(), 1);

    let remaining = storage.sessions.get_one(&id2).await.unwrap().unwrap();
    assert_eq!(remaining.title, "Second Chat");

    let _ = std::fs::remove_file(&path);
}

// ── Test 8: Token records append + read_range + prune ──

#[tokio::test]
async fn test_token_records_append_read_prune() {
    let (storage, path) = create_storage("token-records").await;

    let now = chrono::Utc::now().timestamp();
    let old_ts = now - 100 * 86400; // 100 days ago
    let recent_ts = now - 5 * 86400; // 5 days ago

    let old_record = TokenRecord {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: old_ts,
        user_id: "default".into(),
        agent_id: "default".into(),
        model: "gpt-4o-mini".into(),
        provider: "openai".into(),
        prompt_tokens: 150,
        completion_tokens: 50,
        total_tokens: 200,
        has_tool_calls: true,
        tool_call_count: 2,
        react_rounds: 3,
        success: true,
        latency_ms: 1200,
        estimated_cost_usd: 0.0003,
        trace_id: "trace-old".into(),
    };

    let recent_record = TokenRecord {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: recent_ts,
        user_id: "default".into(),
        agent_id: "analyst".into(),
        model: "deepseek-chat".into(),
        provider: "openai".into(),
        prompt_tokens: 300,
        completion_tokens: 100,
        total_tokens: 400,
        has_tool_calls: false,
        tool_call_count: 0,
        react_rounds: 1,
        success: true,
        latency_ms: 800,
        estimated_cost_usd: 0.0001,
        trace_id: "trace-recent".into(),
    };

    let third_record = TokenRecord {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: now,
        user_id: "default".into(),
        agent_id: "default".into(),
        model: "claude-3".into(),
        provider: "anthropic".into(),
        prompt_tokens: 500,
        completion_tokens: 200,
        total_tokens: 700,
        has_tool_calls: true,
        tool_call_count: 1,
        react_rounds: 2,
        success: false,
        latency_ms: 3000,
        estimated_cost_usd: 0.015,
        trace_id: "trace-now".into(),
    };

    let records = vec![old_record.clone(), recent_record.clone(), third_record.clone()];
    storage
        .stats
        .upsert_batch(&records)
        .await
        .expect("upsert_batch");

    // Read all
    let all = storage
        .stats
        .read_range(None, None)
        .await
        .expect("read_range all");
    assert_eq!(all.len(), 3);

    // Read range: only recent
    let recent = storage
        .stats
        .read_range(Some(now - 10 * 86400), None)
        .await
        .unwrap();
    assert_eq!(recent.len(), 2); // recent_record + third_record
    let agent_ids: Vec<&str> = recent.iter().map(|r| r.agent_id.as_str()).collect();
    assert!(agent_ids.contains(&"analyst"));
    assert!(agent_ids.contains(&"default"));

    // Read range: from old to recent
    let range = storage
        .stats
        .read_range(Some(old_ts), Some(recent_ts))
        .await
        .unwrap();
    assert!(range.len() >= 2);

    // Prune: keep last 30 days — should remove the old record (100 days ago)
    let removed = storage
        .stats
        .prune(30)
        .await
        .expect("prune stats");
    assert_eq!(removed, 1, "should remove 1 old record");

    // Verify old record is gone
    let all = storage.stats.read_range(None, None).await.unwrap();
    assert_eq!(all.len(), 2);
    let ids: Vec<&str> = all.iter().map(|r| r.id.as_str()).collect();
    assert!(!ids.contains(&old_record.id.as_str()));
    assert!(ids.contains(&recent_record.id.as_str()));
    assert!(ids.contains(&third_record.id.as_str()));

    // idempotent upsert (same records should not duplicate)
    storage
        .stats
        .upsert_batch(&[recent_record.clone(), third_record.clone()])
        .await
        .unwrap();
    let all = storage.stats.read_range(None, None).await.unwrap();
    assert_eq!(all.len(), 2, "idempotent upsert should not create duplicates");

    // Prune with keep_days=0 (no-op)
    let removed = storage.stats.prune(0).await.unwrap();
    assert_eq!(removed, 0);

    let _ = std::fs::remove_file(&path);
}
