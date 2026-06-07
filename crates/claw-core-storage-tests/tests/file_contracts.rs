use claw_core_storage_tests::contracts;
use claw_core_storage_tests::backends::helpers;

#[tokio::test]
async fn session() -> anyhow::Result<()> {
    let fs = helpers::file_storage().await;
    contracts::session::run(&fs.storage).await
}

#[tokio::test]
async fn message_log() -> anyhow::Result<()> {
    let fs = helpers::file_storage().await;
    contracts::message_log::run(&fs.storage).await
}

#[tokio::test]
async fn memory() -> anyhow::Result<()> {
    let fs = helpers::file_storage().await;
    contracts::memory::run(&fs.storage).await
}

#[tokio::test]
async fn stats() -> anyhow::Result<()> {
    let fs = helpers::file_storage().await;
    contracts::stats::run(&fs.storage).await
}

#[tokio::test]
async fn tool_cache() -> anyhow::Result<()> {
    let fs = helpers::file_storage().await;
    contracts::tool_cache::run(&fs.storage).await
}

#[tokio::test]
async fn skill() -> anyhow::Result<()> {
    let fs = helpers::file_storage().await;
    contracts::skill::run(&fs.storage).await
}

#[tokio::test]
async fn config_store() -> anyhow::Result<()> {
    use i_rs_claw_core::storage::config_store::ConfigStore;
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::file(dir.path().join("claw"));
    contracts::config_store::run_config(&store).await
}
