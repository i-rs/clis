use std::sync::Arc;
use tempfile::TempDir;
use i_rs_claw_core::storage::ClawStorage;

pub struct FileStorage {
    pub storage: Arc<ClawStorage>,
    pub _dir: TempDir,
}

pub async fn file_storage() -> FileStorage {
    let dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(ClawStorage::file(dir.path().join("claw")));
    FileStorage {
        storage,
        _dir: dir,
    }
}

pub async fn sqlite_storage() -> (Arc<ClawStorage>, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let storage = Arc::new(ClawStorage::sqlite(path).await.unwrap());
    (storage, dir)
}

pub async fn mysql_storage() -> ClawStorage {
    let url = std::env::var("MYSQL_URL")
        .unwrap_or("mysql://root:test@localhost:3306/claw_test".into());
    ClawStorage::mysql(&url).await.unwrap()
}

pub async fn postgres_storage() -> ClawStorage {
    let url = std::env::var("PG_URL")
        .unwrap_or("postgres://postgres:test@localhost:5432/claw_test".into());
    ClawStorage::postgres(&url).await.unwrap()
}
