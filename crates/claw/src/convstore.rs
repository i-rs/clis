use std::path::PathBuf;
use std::sync::Arc;

use i_rs_claw_core::storage::ClawStorage;
use i_rs_claw_core::storage::SearchResult;

/// Full-text search across all conversation session messages.
///
/// Delegates to the active storage backend via `MessageLog::search`.
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
        i_rs_claw_core::utils::sync_block_on(async move {
            storage
                .message_log
                .search(&query, max_results)
                .await
                .unwrap_or_default()
        })
    }
}
