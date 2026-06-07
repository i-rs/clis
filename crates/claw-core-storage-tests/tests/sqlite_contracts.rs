use claw_core_storage_tests::contracts;
use claw_core_storage_tests::backends::helpers;

#[tokio::test]
async fn session() -> anyhow::Result<()> {
    let (storage, _dir) = helpers::sqlite_storage().await;
    contracts::session::run(&storage).await
}

#[tokio::test]
async fn message_log() -> anyhow::Result<()> {
    let (storage, _dir) = helpers::sqlite_storage().await;
    contracts::message_log::run(&storage).await
}

#[tokio::test]
async fn memory() -> anyhow::Result<()> {
    let (storage, _dir) = helpers::sqlite_storage().await;
    contracts::memory::run(&storage).await
}

#[tokio::test]
async fn stats() -> anyhow::Result<()> {
    let (storage, _dir) = helpers::sqlite_storage().await;
    contracts::stats::run(&storage).await
}

#[tokio::test]
async fn tool_cache() -> anyhow::Result<()> {
    let (storage, _dir) = helpers::sqlite_storage().await;
    contracts::tool_cache::run(&storage).await
}

#[tokio::test]
async fn skill() -> anyhow::Result<()> {
    let (storage, _dir) = helpers::sqlite_storage().await;
    contracts::skill::run(&storage).await
}

#[tokio::test]
async fn config_store() -> anyhow::Result<()> {
    use i_rs_claw_core::storage::sql::sqlite::SqliteBackend;
    let dir = tempfile::tempdir().unwrap();
    let backend = SqliteBackend::new(dir.path().join("test.db")).await.unwrap();
    let store = backend.into_config_store();
    contracts::config_store::run_config(&store).await
}
