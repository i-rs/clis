use std::sync::{Arc, RwLock};

/// A thread-safe, in-memory store backed by a JSON file.
///
/// - Data stays in memory (read lock = zero I/O)
/// - Writes flush to disk via i_rs_core::Storage
/// - No file lock contention — only one process-wide write per disk flush
/// - No spawn_blocking needed — RwLock is async-friendly
pub struct SharedStore<T> {
    inner: Arc<RwLock<T>>,
    filename: String,
}

impl<T> Clone for SharedStore<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            filename: self.filename.clone(),
        }
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned + Default> SharedStore<T> {
    /// Create a SharedStore and load existing data from disk.
    /// If the file doesn't exist, it starts with a default (empty) store.
    pub fn load(filename: &str) -> Self {
        let mut storage = i_rs_core::Storage::<T>::new(filename);
        // Ignore load errors — if file doesn't exist or is corrupt, use default
        let _ = storage.load();
        let inner = Arc::new(RwLock::new(storage.data));
        Self {
            inner,
            filename: filename.to_string(),
        }
    }

    /// Read data under a read lock.
    /// Multiple readers can proceed concurrently.
    pub fn read<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.inner.read().expect("SharedStore read lock poisoned");
        f(&guard)
    }

    /// Write data under a write lock, then flush to disk.
    /// Only one writer at a time.
    pub fn write<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.inner.write().expect("SharedStore write lock poisoned");
        let result = f(&mut guard);
        // Flush to disk after every mutation
        let storage = i_rs_core::Storage::<T>::new(&self.filename);
        if let Err(e) = storage.save_data(&guard) {
            eprintln!("[SharedStore] Failed to flush {}: {e}", self.filename);
        }
        result
    }

    /// Export all data as pretty JSON (for the data export endpoint).
    pub fn export_json(&self) -> String {
        self.read(|data| {
            serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_string())
        })
    }

    /// Import data from a JSON string, replacing all contents.
    pub fn import_json(&self, json_str: &str) -> Result<(), String> {
        let data: T =
            serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {e}"))?;
        self.write(|store| *store = data);
        Ok(())
    }

    /// Clear all data (reset to default/empty state).
    pub fn clear(&self) {
        self.write(|store| *store = T::default());
    }
}
