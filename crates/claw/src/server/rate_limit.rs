use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Global concurrent chat request limiter.
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
    inner: std::sync::Arc<ChatConcurrency>,
    released: AtomicBool,
}

impl ConcurrencyGuard {
    pub fn new(inner: std::sync::Arc<ChatConcurrency>) -> Self {
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
