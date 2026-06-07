use i_rs_claw_core::storage::mongo::MongoBackend;
use claw_core_storage_tests::contracts;

async fn mongo_storage() -> std::sync::Arc<i_rs_claw_core::storage::ClawStorage> {
    let url = std::env::var("MONGO_URL")
        .unwrap_or("mongodb://localhost:27017".into());
    let db = std::env::var("MONGO_DB").unwrap_or("claw_test".into());
    std::sync::Arc::new(i_rs_claw_core::storage::ClawStorage::mongo(&url, &db).await.unwrap())
}

#[ignore = "requires: docker compose up mongo"]
#[tokio::test]
async fn session() -> anyhow::Result<()> {
    let storage = mongo_storage().await;
    contracts::session::run(&storage).await
}

#[ignore = "requires: docker compose up mongo"]
#[tokio::test]
async fn message_log() -> anyhow::Result<()> {
    let storage = mongo_storage().await;
    contracts::message_log::run(&storage).await
}

#[ignore = "requires: docker compose up mongo"]
#[tokio::test]
async fn memory() -> anyhow::Result<()> {
    let storage = mongo_storage().await;
    contracts::memory::run(&storage).await
}

#[ignore = "requires: docker compose up mongo"]
#[tokio::test]
async fn stats() -> anyhow::Result<()> {
    let storage = mongo_storage().await;
    contracts::stats::run(&storage).await
}

#[ignore = "requires: docker compose up mongo"]
#[tokio::test]
async fn tool_cache() -> anyhow::Result<()> {
    let storage = mongo_storage().await;
    contracts::tool_cache::run(&storage).await
}

#[ignore = "requires: docker compose up mongo"]
#[tokio::test]
async fn skill() -> anyhow::Result<()> {
    let storage = mongo_storage().await;
    contracts::skill::run(&storage).await
}

#[ignore = "requires: docker compose up mongo"]
#[tokio::test]
async fn config_store() -> anyhow::Result<()> {
    let url = std::env::var("MONGO_URL")
        .unwrap_or("mongodb://localhost:27017".into());
    let db = std::env::var("MONGO_DB").unwrap_or("claw_test".into());
    let backend = MongoBackend::new(&url, &db).await.unwrap();
    let store = backend.into_config_store();
    contracts::config_store::run_config(&store).await
}
