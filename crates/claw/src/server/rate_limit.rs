use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

/// Concurrent chat request limiter for a single scope (e.g. one user).
pub struct ChatConcurrency {
    active: AtomicU32,
    max: u32,
}

impl ChatConcurrency {
    pub fn new(max: u32) -> Self {
        Self {
            active: AtomicU32::new(0),
            max,
        }
    }

    pub fn try_acquire(&self) -> Result<(), String> {
        let current = self.active.fetch_add(1, Ordering::AcqRel);
        if current >= self.max {
            self.active.fetch_sub(1, Ordering::AcqRel);
            return Err(format!(
                "Rate limit: max {} concurrent chat requests",
                self.max
            ));
        }
        Ok(())
    }

    pub fn release(&self) {
        self.active.fetch_sub(1, Ordering::AcqRel);
    }
}

#[allow(dead_code)]
impl ChatConcurrency {
    pub fn active_count(&self) -> u32 {
        self.active.load(Ordering::Acquire)
    }
}

/// RAII guard that releases the concurrency slot on drop if not already released.
pub struct ConcurrencyGuard {
    inner: Arc<ChatConcurrency>,
    released: AtomicBool,
}

impl ConcurrencyGuard {
    pub fn new(inner: Arc<ChatConcurrency>) -> Self {
        Self { inner, released: AtomicBool::new(false) }
    }

    /// Release the slot explicitly (idempotent).
    pub fn release(&self) {
        if !self.released.swap(true, Ordering::Relaxed) {
            self.inner.release();
        }
    }
}

impl Drop for ConcurrencyGuard {
    fn drop(&mut self) {
        self.release();
    }
}

/// Per-user concurrency limiter.
///
/// Each user gets an independent [`ChatConcurrency`] with the same `max`,
/// so one user cannot exhaust the budget of another.
pub struct UserConcurrencyLimiter {
    max_per_user: u32,
    limiters: Mutex<HashMap<String, Arc<ChatConcurrency>>>,
}

impl UserConcurrencyLimiter {
    pub fn new(max_per_user: u32) -> Self {
        Self {
            max_per_user,
            limiters: Mutex::new(HashMap::new()),
        }
    }

    /// Try to acquire a concurrency slot for `user_id`.
    ///
    /// On success, returns a [`ConcurrencyGuard`] that releases the slot on drop.
    /// On failure, returns an error message suitable for an HTTP 429 response.
    pub fn try_acquire_for(&self, user_id: &str) -> Result<ConcurrencyGuard, String> {
        let limiter = {
            let mut map = self.limiters.lock().unwrap_or_else(|e| e.into_inner());
            map.entry(user_id.to_string())
                .or_insert_with(|| Arc::new(ChatConcurrency::new(self.max_per_user)))
                .clone()
        };
        limiter.try_acquire()?;
        Ok(ConcurrencyGuard::new(limiter))
    }
}
