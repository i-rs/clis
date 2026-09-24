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
        Self {
            inner,
            released: AtomicBool::new(false),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_acquire_under_limit() {
        let limiter = ChatConcurrency::new(2);
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_ok());
    }

    #[test]
    fn test_try_acquire_over_limit() {
        let limiter = ChatConcurrency::new(2);
        limiter.try_acquire().unwrap(); // 1
        limiter.try_acquire().unwrap(); // 2
        assert!(limiter.try_acquire().is_err()); // 3
    }

    #[test]
    fn test_acquire_over_limit_rolls_back() {
        let limiter = ChatConcurrency::new(1);
        limiter.try_acquire().unwrap(); // counter = 1
        assert!(limiter.try_acquire().is_err()); // increments to 2, then rolls back to 1
        limiter.release(); // counter = 0
        assert!(limiter.try_acquire().is_ok()); // should succeed now
    }

    #[test]
    fn test_guard_release_idempotent() {
        let cc = Arc::new(ChatConcurrency::new(1));
        cc.active.store(1, std::sync::atomic::Ordering::Release);
        let guard = ConcurrencyGuard::new(cc.clone());
        guard.release();
        guard.release(); // idempotent — should not underflow
        assert_eq!(cc.active_count(), 0);
    }

    #[test]
    fn test_guard_drop_releases() {
        let cc = Arc::new(ChatConcurrency::new(1));
        cc.active.store(1, Ordering::Release);
        let guard = ConcurrencyGuard::new(cc.clone());
        drop(guard);
        assert_eq!(cc.active_count(), 0);
    }

    #[test]
    fn test_guard_explicit_then_drop_idempotent() {
        let cc = Arc::new(ChatConcurrency::new(1));
        cc.active.store(1, Ordering::Release);
        let guard = ConcurrencyGuard::new(cc.clone());
        guard.release(); // explicit
        assert_eq!(cc.active_count(), 0);
        drop(guard); // drop should be idempotent (no double-release)
        assert_eq!(cc.active_count(), 0);
    }

    #[test]
    fn test_user_concurrency_different_users_independent() {
        let limiter = UserConcurrencyLimiter::new(1);
        let _g1 = limiter
            .try_acquire_for("alice")
            .expect("alice should acquire");
        // Bob should still be able to acquire (different user, own limit)
        let g2 = limiter.try_acquire_for("bob").expect("bob should acquire");
        drop(g2);
        // Alice still holds hers
        assert!(limiter.try_acquire_for("alice").is_err());
    }

    #[test]
    fn test_user_concurrency_same_user_blocked() {
        let limiter = UserConcurrencyLimiter::new(1);
        let _g = limiter
            .try_acquire_for("alice")
            .expect("alice should acquire");
        assert!(limiter.try_acquire_for("alice").is_err());
    }

    #[test]
    fn test_user_concurrency_release_frees_slot() {
        let limiter = UserConcurrencyLimiter::new(1);
        {
            let _g = limiter
                .try_acquire_for("alice")
                .expect("alice should acquire");
        }
        // Slot freed, should be able to acquire again
        assert!(limiter.try_acquire_for("alice").is_ok());
    }

    #[test]
    fn test_active_count_reflects_state() {
        let cc = Arc::new(ChatConcurrency::new(2));
        cc.try_acquire().unwrap(); // 1
        cc.try_acquire().unwrap(); // 2
        assert_eq!(cc.active_count(), 2);
        cc.release(); // back to 1
        assert_eq!(cc.active_count(), 1);
    }
}
