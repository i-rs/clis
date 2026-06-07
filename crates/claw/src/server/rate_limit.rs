use std::sync::atomic::{AtomicU32, Ordering};

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
