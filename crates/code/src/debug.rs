use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct HttpLogEntry {
    pub url: String,
    pub request_body: String,
    pub response_status: u16,
    pub response_body_preview: String,
    pub duration_ms: u64,
    pub timestamp: String,
}

impl HttpLogEntry {
    pub fn status_label(&self) -> String {
        match self.response_status {
            200 => "200 OK".into(),
            401 => "401 Unauthorized".into(),
            s if s >= 400 => format!("{} Error", s),
            s => format!("{}", s),
        }
    }

    pub fn path(&self) -> String {
        self.url.split("/v1/").nth(1).unwrap_or(&self.url).to_string()
    }

    pub fn time_short(&self) -> &str {
        if self.timestamp.len() > 19 {
            &self.timestamp[11..19]
        } else {
            &self.timestamp
        }
    }
}

static HTTP_LOG: Mutex<Vec<HttpLogEntry>> = Mutex::new(Vec::new());

pub fn push_log(entry: HttpLogEntry) {
    if let Ok(mut log) = HTTP_LOG.lock() {
        log.push(entry);
        if log.len() > 200 {
            log.remove(0);
        }
    }
}

pub fn get_log() -> Vec<HttpLogEntry> {
    if let Ok(log) = HTTP_LOG.lock() {
        log.clone()
    } else {
        Vec::new()
    }
}

pub fn clear_log() {
    if let Ok(mut log) = HTTP_LOG.lock() {
        log.clear();
    }
}

pub fn log_count() -> usize {
    if let Ok(log) = HTTP_LOG.lock() {
        log.len()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_log_and_get_log() {
        clear_log();
        let entry = HttpLogEntry {
            url: "https://example.com".into(),
            request_body: "{}".into(),
            response_status: 200,
            response_body_preview: "ok".into(),
            duration_ms: 100,
            timestamp: "2024-01-01".into(),
        };
        push_log(entry);
        let logs = get_log();
        assert!(!logs.is_empty(), "should have at least one log entry");
        assert_eq!(logs[0].url, "https://example.com");
    }

    #[test]
    fn test_clear_log() {
        clear_log();
        push_log(HttpLogEntry {
            url: "test".into(),
            request_body: String::new(),
            response_status: 200,
            response_body_preview: String::new(),
            duration_ms: 0,
            timestamp: String::new(),
        });
        assert!(log_count() > 0, "should have entries before clear");
        clear_log();
        assert_eq!(log_count(), 0, "should be empty after clear");
    }
}
