use i_rs_claw_core::storage::config_store::{
    AgentConfigRow, ConfigStore, DashboardUserRow, McpServerConfigRow, ProviderConfigRow,
};

pub async fn run_config(store: &ConfigStore) -> anyhow::Result<()> {
    let now = chrono::Utc::now().timestamp();

    // 1. Agent config CRUD
    let agent_row = AgentConfigRow {
        user_id: "default".into(),
        agent_id: "test-agent".into(),
        provider_ref: None,
        provider: "openai".into(),
        api_key: "sk-test".into(),
        base_url: String::new(),
        model: "gpt-4".into(),
        enabled_tools: vec![],
        system_prompt: "You are a test".into(),
        system_prompt_file: None,
        capabilities: vec![],
        execution_mode: "auto".into(),
        created_at: now,
        updated_at: now,
    };
    store.agent_configs.upsert(&agent_row).await?;
    let agents = store.agent_configs.load_all("default").await?;
    assert_eq!(agents.len(), 1, "should have 1 agent config");
    assert_eq!(agents[0].system_prompt, "You are a test");
    store.agent_configs.delete("default", "test-agent").await?;
    let after = store.agent_configs.load_all("default").await?;
    assert_eq!(after.len(), 0, "agent config should be deleted");

    // 2. Provider config CRUD
    let provider_row = ProviderConfigRow {
        name: "test-provider".into(),
        provider: "openai".into(),
        api_key: "sk-test".into(),
        base_url: String::new(),
        model: "gpt-4".into(),
        created_at: now,
        updated_at: now,
    };
    store.provider_configs.upsert(&provider_row).await?;
    let providers = store.provider_configs.load_all().await?;
    assert_eq!(providers.len(), 1, "should have 1 provider config");
    store.provider_configs.delete("test-provider").await?;
    let after_p = store.provider_configs.load_all().await?;
    assert_eq!(after_p.len(), 0, "provider config should be deleted");

    // 3. Dashboard user CRUD
    let user_row = DashboardUserRow {
        user_id: "test-user".into(),
        token_hash: "abc123".into(),
        display_name: "Test".into(),
        created_at: now,
        updated_at: now,
    };
    store.dashboard_users.upsert(&user_row).await?;
    let users = store.dashboard_users.load_all().await?;
    assert_eq!(users.len(), 1, "should have 1 dashboard user");
    let found = store.dashboard_users.find_by_token_hash("abc123").await?;
    assert!(found.is_some(), "should find user by token hash");
    store.dashboard_users.delete("test-user").await?;
    assert!(
        store.dashboard_users.load_all().await?.is_empty(),
        "users should be empty after delete"
    );

    // 4. MCP server config CRUD
    let mcp_row = McpServerConfigRow {
        user_id: "default".into(),
        agent_id: None,
        name: "test-mcp".into(),
        transport_type: "stdio".into(),
        command: Some("echo".into()),
        args_json: None,
        url: None,
        env_json: None,
        enabled: true,
    };
    store.mcp_servers.upsert(&mcp_row).await?;
    let mcps = store.mcp_servers.load_for("default", None).await?;
    assert_eq!(mcps.len(), 1, "should have 1 mcp server config");
    store
        .mcp_servers
        .delete("default", None, "test-mcp")
        .await?;
    assert!(
        store
            .mcp_servers
            .load_for("default", None)
            .await?
            .is_empty(),
        "mcp configs should be empty after delete"
    );

    // 5. App settings CRUD
    store
        .app_settings
        .set("theme", &serde_json::json!("dark"))
        .await?;
    let loaded = store.app_settings.get("theme").await?;
    assert_eq!(loaded, Some(serde_json::json!("dark")));
    let all_settings = store.app_settings.load_all().await?;
    assert_eq!(all_settings.len(), 1, "should have 1 setting");

    Ok(())
}
