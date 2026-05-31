use std::path::PathBuf;
use std::sync::Arc;

use crate::storage::ClawStorage;
use crate::storage::SearchResult;

/// Full-text search across all conversation session JSONL files.
///
/// Delegates to the active storage backend via `MessageRepo::search()`.
pub struct ConvStore {
    storage: Arc<ClawStorage>,
}

impl ConvStore {
    /// Create with a specific storage backend (for DI/testing).
    pub fn new(storage: Arc<ClawStorage>) -> Self {
        Self { storage }
    }

    /// Create with file backend at the given directory (backward-compatible).
    pub fn for_claw_dir(claw_dir: PathBuf) -> Self {
        Self::new(Arc::new(ClawStorage::file(claw_dir)))
    }

    /// Search message text across all sessions.
    ///
    /// `query` is matched case-insensitively against user, assistant, and error
    /// message text, as well as tool_call names.
    /// Returns up to `max_results` results.
    pub fn search(&self, query: &str, max_results: usize) -> Vec<SearchResult> {
        let storage = self.storage.clone();
        let query = query.to_string();
        // Use block-on bridge — caller may not be in an async context
        match tokio::runtime::Handle::try_current() {
            Ok(h) => h.block_on(async move {
                storage
                    .messages
                    .search(&query, max_results)
                    .await
                    .unwrap_or_default()
            }),
            Err(_) => tokio::runtime::Runtime::new()
                .expect("ConvStore: failed to create temp runtime")
                .block_on(async move {
                    storage
                        .messages
                        .search(&query, max_results)
                        .await
                        .unwrap_or_default()
                }),
        }
    }
}
