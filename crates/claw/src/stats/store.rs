#![allow(dead_code)]

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Mutex;

use super::TokenRecord;

/// Guards sequential access to JSONL append operations.
/// Prevents interleaved writes when multiple sessions/gateways
/// append concurrently. `O_APPEND` is atomic per-write on macOS,
/// but batch writes or concurrent prune+append could corrupt.
static WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Number of records to keep in the index for range queries.
#[allow(dead_code)]
const INDEX_INTERVAL: usize = 100;

/// Lightweight line-offset index for timestamp-range queries.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub(crate) struct StoreIndex {
    /// (file_offset, timestamp) entries, one per INDEX_INTERVAL lines.
    entries: Vec<(u64, i64)>,
    /// Total number of lines indexed.
    total_lines: usize,
}

#[allow(dead_code)]
impl StoreIndex {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            total_lines: 0,
        }
    }

    /// Rebuild the index from scratch by scanning the file.
    pub fn rebuild(&mut self, path: &Path) -> std::io::Result<()> {
        self.entries.clear();
        self.total_lines = 0;

        if !path.exists() {
            return Ok(());
        }

        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(&file);
        let mut line_num = 0usize;
        let mut offset = 0u64;

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                offset += (line.len() + 1) as u64;
                continue;
            }
            if line_num.is_multiple_of(INDEX_INTERVAL)
                && let Some(ts) = extract_timestamp(&line)
            {
                self.entries.push((offset, ts));
            }
            offset += (line.len() + 1) as u64;
            line_num += 1;
        }
        self.total_lines = line_num;

        Ok(())
    }

    /// Get the starting file offset for a timestamp threshold.
    /// Returns None if no records are newer than `min_ts`.
    pub fn start_offset(&self, min_ts: i64) -> Option<u64> {
        // Binary search for the first entry with timestamp >= min_ts
        let idx = self.entries.partition_point(|e| e.1 < min_ts);
        self.entries.get(idx).map(|e| e.0)
    }
}

/// Extract timestamp from a JSON line: {"id":"...","timestamp":1234567890,...}
fn extract_timestamp(line: &str) -> Option<i64> {
    // Quick scan for "timestamp":NUMBER
    let marker = "\"timestamp\":";
    line.find(marker).and_then(|pos| {
        let rest = &line[pos + marker.len()..];
        let end = rest.find(|c: char| !c.is_ascii_digit() && c != '-')?;
        rest[..end].parse::<i64>().ok()
    })
}

/// Append a single TokenRecord as a JSON line to the store file.
#[allow(dead_code)]
pub(crate) fn append_record(path: &Path, record: &TokenRecord) -> std::io::Result<()> {
    let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string(record)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", json)?;
    Ok(())
}

/// Append multiple records in batch.
pub(crate) fn append_records(path: &Path, records: &[TokenRecord]) -> std::io::Result<()> {
    if records.is_empty() {
        return Ok(());
    }

    let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    for record in records {
        let json = serde_json::to_string(record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        writeln!(file, "{}", json)?;
    }
    Ok(())
}

/// Read records within a timestamp range [from, to] inclusive.
/// If `from` is None, reads from the beginning.
/// If `to` is None, reads until the end.
pub(crate) fn read_range(
    path: &Path,
    from: Option<i64>,
    to: Option<i64>,
) -> std::io::Result<Vec<TokenRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(&file);
    let mut records = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        // Quick timestamp filter before deserialization
        if let Some(ts) = extract_timestamp(&line) {
            if let Some(f) = from
                && ts < f
            {
                continue;
            }
            if let Some(t) = to
                && ts > t
            {
                continue;
            }
        }

        match serde_json::from_str::<TokenRecord>(&line) {
            Ok(record) => records.push(record),
            Err(e) => {
                // Log and skip corrupted lines
                tracing::warn!("跳过损坏的 token 记录行: {}", e);
            }
        }
    }

    Ok(records)
}

/// Remove records older than `keep_days` days.
/// Reads the full file, filters out old records, and rewrites the file.
/// Returns the number of removed records.
pub(crate) fn prune_old_records(path: &Path, keep_days: u32) -> std::io::Result<usize> {
    if keep_days == 0 || !path.exists() {
        return Ok(0);
    }

    let cutoff = chrono::Local::now().timestamp() - (keep_days as i64 * 86400);
    let all = read_range(path, None, None)?;

    let before = all.len();
    let kept: Vec<&TokenRecord> = all.iter().filter(|r| r.timestamp >= cutoff).collect();
    let removed = before - kept.len();

    if removed == 0 {
        return Ok(0);
    }

    // Rewrite the file with only kept records
    let temp_path = path.with_extension("jsonl.tmp");
    {
        let mut file = std::fs::File::create(&temp_path)?;
        for record in &kept {
            let json = serde_json::to_string(record)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            writeln!(file, "{}", json)?;
        }
    }
    std::fs::rename(&temp_path, path)?;

    tracing::info!("清理了 {} 条过期统计记录", removed);
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use i_rs_claw_core::stats::TokenRecord;

    #[test]
    fn test_extract_timestamp() {
        assert_eq!(
            extract_timestamp(r#"{"id":"abc","timestamp":1716220800,"agent_id":"default"}"#),
            Some(1716220800)
        );
        assert_eq!(extract_timestamp(r#"{"timestamp":-1}"#), Some(-1));
        assert_eq!(extract_timestamp(r#"{}"#), None);
        assert_eq!(extract_timestamp(r#""no objects""#), None);
    }

    #[test]
    fn test_append_and_read() {
        let dir = std::env::temp_dir().join(format!("stats_test_{}", std::process::id()));
        let path = dir.join("usage.jsonl");
        let _ = std::fs::remove_dir_all(&dir);

        let record = TokenRecord {
            id: "test-1".to_string(),
            timestamp: 1716220800,
            agent_id: "default".to_string(),
            model: "gpt-4o-mini".to_string(),
            provider: "openai".to_string(),
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            has_tool_calls: false,
            tool_call_count: 0,
            react_rounds: 1,
            success: true,
            latency_ms: 500,
            estimated_cost_usd: 0.0001,
            trace_id: String::new(),
        };

        append_record(&path, &record).unwrap();

        let records = read_range(&path, None, None).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "test-1");

        let records = read_range(&path, Some(1716220801), None).unwrap();
        assert_eq!(records.len(), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
