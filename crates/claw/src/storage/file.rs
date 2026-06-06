//! File-system storage backend.
//!
//! Mirrors the original on-disk layout under `~/.i-rs/claw/`:
//!
//! ```text
//! {claw_dir}/
//! ├── index.json
//! ├── sessions/{id}.jsonl / {id}_api.json / {id}_plan.json
//! ├── agents/{agent_id}/memory.json / hot_docs_cache.json / skills/*.md
//! └── stats/usage.jsonl
//! ```
//!
//! Each repository trait is implemented by a dedicated zero-sized wrapper around
//! `PathBuf`, avoiding the method-ambiguity problem of a monolithic `FileBackend`.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::*;

/// Domain-specific locks to prevent unrelated writes from blocking each other.
static MESSAGES_LOCK: Mutex<()> = Mutex::new(());
static STATS_LOCK: Mutex<()> = Mutex::new(());
static SESSION_LOCK: Mutex<()> = Mutex::new(());

fn ensure_dir(p: &Path) -> anyhow::Result<()> {
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    crate::utils::atomic_write(path, content)
}

async fn blocking<F, T>(f: F) -> anyhow::Result<T>
where
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| anyhow::anyhow!("blocking task panicked: {}", e))?
}

fn lock_guard(mu: &Mutex<()>) -> std::sync::MutexGuard<'_, ()> {
    mu.lock().unwrap_or_else(|e| {
        tracing::error!("file mutex poisoned — previous write may have panicked; recovering");
        e.into_inner()
    })
}

// ═══════════════════════════════════════════════════════════════════
//  Path helpers (shared)
// ═══════════════════════════════════════════════════════════════════

fn index_path(claw_dir: &Path) -> PathBuf {
    claw_dir.join("index.json")
}

fn sessions_dir(claw_dir: &Path) -> PathBuf {
    claw_dir.join("sessions")
}

fn messages_path(claw_dir: &Path, session_id: &str) -> PathBuf {
    sessions_dir(claw_dir).join(format!("{}.jsonl", session_id))
}

fn api_cache_path(claw_dir: &Path, session_id: &str) -> PathBuf {
    sessions_dir(claw_dir).join(format!("{}_api.json", session_id))
}

fn plan_steps_path(claw_dir: &Path, session_id: &str) -> PathBuf {
    sessions_dir(claw_dir).join(format!("{}_plan.json", session_id))
}

fn agent_dir(claw_dir: &Path, agent_id: &str) -> PathBuf {
    claw_dir.join("agents").join(agent_id)
}

fn memory_path(claw_dir: &Path, agent_id: &str) -> PathBuf {
    agent_dir(claw_dir, agent_id).join("memory.json")
}

fn skills_dir(claw_dir: &Path, agent_id: &str) -> PathBuf {
    agent_dir(claw_dir, agent_id).join("skills")
}

fn tool_cache_path(claw_dir: &Path, agent_id: &str) -> PathBuf {
    agent_dir(claw_dir, agent_id).join("hot_docs_cache.json")
}

fn stats_path(claw_dir: &Path) -> PathBuf {
    claw_dir.join("stats").join("usage.jsonl")
}

/// Extract timestamp from a JSONL line by parsing the whole line.
/// Avoids substring matching (which could match `"start_timestamp"` etc.).
fn extract_timestamp(line: &str) -> Option<i64> {
    serde_json::from_str::<serde_json::Value>(line.trim())
        .ok()
        .and_then(|v| v.get("timestamp").and_then(|t| t.as_i64()))
}

// ═══════════════════════════════════════════════════════════════════
//  Individual file stores
// ═══════════════════════════════════════════════════════════════════

// ── SessionRepo ──

#[derive(Clone)]
pub struct FileSessionStore {
    claw_dir: PathBuf,
}

impl FileSessionStore {
    pub(crate) fn new(claw_dir: PathBuf) -> Self {
        Self { claw_dir }
    }
}

#[async_trait]
impl SessionRepo for FileSessionStore {
    async fn load_all(&self) -> anyhow::Result<Vec<crate::session::SessionMeta>> {
        let path = index_path(&self.claw_dir);
        blocking(move || {
            if !path.exists() {
                return Ok(Vec::new());
            }
            let content = std::fs::read_to_string(&path)?;
            serde_json::from_str(&content)
                .inspect_err(|e| tracing::error!("index.json 损坏: {} — 不会静默清空", e))
                .map_err(|e| anyhow::anyhow!("index.json 损坏: {}", e))
        })
        .await
    }

    async fn save_all(&self, sessions: &[crate::session::SessionMeta]) -> anyhow::Result<()> {
        let path = index_path(&self.claw_dir);
        let content = serde_json::to_string_pretty(sessions)?;
        blocking(move || {
            ensure_dir(&path)?;
            // 备份旧文件 (如果存在)
            if path.exists() {
                let bak = path.with_extension("json.bak");
                std::fs::copy(&path, &bak).ok();
            }
            atomic_write(&path, &content).map_err(anyhow::Error::from)
        })
        .await
    }

    async fn get_one(&self, id: &str) -> anyhow::Result<Option<crate::session::SessionMeta>> {
        let all = self.load_all().await?;
        Ok(all.into_iter().find(|s| s.id == id))
    }

    async fn upsert(&self, session: &crate::session::SessionMeta) -> anyhow::Result<()> {
        let session = session.clone();
        let claw_dir = self.claw_dir.clone();
        blocking(move || {
            let _lock = lock_guard(&SESSION_LOCK);
            let mut all = session_io::load_sessions_sync(&claw_dir)?;
            if let Some(pos) = all.iter().position(|s| s.id == session.id) {
                all[pos] = session;
            } else {
                all.push(session);
            }
            session_io::save_sessions_sync(&claw_dir, &all)
        })
        .await
    }

    async fn delete_one(&self, id: &str) -> anyhow::Result<()> {
        let id = id.to_string();
        let claw_dir = self.claw_dir.clone();
        blocking(move || {
            let _lock = lock_guard(&SESSION_LOCK);
            let mut all = session_io::load_sessions_sync(&claw_dir)?;
            all.retain(|s| s.id != id);
            session_io::save_sessions_sync(&claw_dir, &all)
        })
        .await
    }

    async fn count(&self) -> anyhow::Result<usize> {
        Ok(self.load_all().await?.len())
    }
}

// ── ApiCacheRepo ──

#[derive(Clone)]
pub struct FileApiCacheStore {
    claw_dir: PathBuf,
}

impl FileApiCacheStore {
    pub(crate) fn new(claw_dir: PathBuf) -> Self {
        Self { claw_dir }
    }
}

#[async_trait]
impl ApiCacheRepo for FileApiCacheStore {
    async fn save(&self, session_id: &str, messages: &[serde_json::Value]) -> anyhow::Result<()> {
        let path = api_cache_path(&self.claw_dir, session_id);
        let content = serde_json::to_string(messages)?;
        blocking(move || {
            ensure_dir(&path)?;
            atomic_write(&path, &content).map_err(anyhow::Error::from)
        })
        .await
    }

    async fn load(&self, session_id: &str) -> anyhow::Result<Option<Vec<serde_json::Value>>> {
        let path = api_cache_path(&self.claw_dir, session_id);
        blocking(move || {
            if !path.exists() {
                return Ok(None);
            }
            let content = std::fs::read_to_string(&path)?;
            let data = serde_json::from_str(&content)
                .inspect_err(|e| {
                    tracing::error!("api_cache 文件损坏 ({}): {}", path.display(), e)
                })?;
            Ok(Some(data))
        })
        .await
    }

    async fn delete(&self, session_id: &str) -> anyhow::Result<()> {
        let path = api_cache_path(&self.claw_dir, session_id);
        blocking(move || {
            let _ = std::fs::remove_file(&path);
            Ok(())
        })
        .await
    }
}

// ── PlanStepsRepo ──

#[derive(Clone)]
pub struct FilePlanStepsStore {
    claw_dir: PathBuf,
}

impl FilePlanStepsStore {
    pub(crate) fn new(claw_dir: PathBuf) -> Self {
        Self { claw_dir }
    }
}

#[async_trait]
impl PlanStepsRepo for FilePlanStepsStore {
    async fn save(&self, session_id: &str, steps: &[crate::app::PlanStep]) -> anyhow::Result<()> {
        let path = plan_steps_path(&self.claw_dir, session_id);
        let content = serde_json::to_string(steps)?;
        blocking(move || {
            ensure_dir(&path)?;
            atomic_write(&path, &content).map_err(anyhow::Error::from)
        })
        .await
    }

    async fn load(&self, session_id: &str) -> anyhow::Result<Vec<crate::app::PlanStep>> {
        let path = plan_steps_path(&self.claw_dir, session_id);
        blocking(move || {
            if !path.exists() {
                return Ok(Vec::new());
            }
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        })
        .await
    }

    async fn delete(&self, session_id: &str) -> anyhow::Result<()> {
        let path = plan_steps_path(&self.claw_dir, session_id);
        blocking(move || {
            let _ = std::fs::remove_file(&path);
            Ok(())
        })
        .await
    }
}

// ── MemoryRepo ──

#[derive(Clone)]
pub struct FileMemoryStore {
    claw_dir: PathBuf,
}

impl FileMemoryStore {
    pub(crate) fn new(claw_dir: PathBuf) -> Self {
        Self { claw_dir }
    }
}

#[async_trait]
impl MemoryRepo for FileMemoryStore {
    async fn load(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<Option<crate::memory::CrossSessionMemory>> {
        let path = memory_path(&self.claw_dir, agent_id);
        blocking(move || {
            if !path.exists() {
                return Ok(None);
            }
            Ok(Some(crate::memory::CrossSessionMemory::load_from(&path)))
        })
        .await
    }

    async fn save(
        &self,
        agent_id: &str,
        memory: &crate::memory::CrossSessionMemory,
    ) -> anyhow::Result<()> {
        let path = memory_path(&self.claw_dir, agent_id);
        let content = serde_json::to_string_pretty(memory)?;
        blocking(move || {
            ensure_dir(&path)?;
            atomic_write(&path, &content).map_err(anyhow::Error::from)
        })
        .await
    }
}

// ── StatsRepo ──

#[derive(Clone)]
pub struct FileStatsStore {
    claw_dir: PathBuf,
}

impl FileStatsStore {
    pub(crate) fn new(claw_dir: PathBuf) -> Self {
        Self { claw_dir }
    }
}

#[async_trait]
impl StatsRepo for FileStatsStore {
    async fn upsert_batch(&self, records: &[crate::stats::TokenRecord]) -> anyhow::Result<()> {
        let path = stats_path(&self.claw_dir);
        let json_lines: Vec<String> = records
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?;

        blocking(move || {
            let _guard = lock_guard(&STATS_LOCK);
            ensure_dir(&path)?;
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            for line in &json_lines {
                writeln!(file, "{}", line)?;
            }
            Ok(())
        })
        .await
    }

    async fn read_range(
        &self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> anyhow::Result<Vec<crate::stats::TokenRecord>> {
        let path = stats_path(&self.claw_dir);
        blocking(move || read_range_sync(&path, from, to)).await
    }

    async fn prune(&self, keep_days: u32) -> anyhow::Result<usize> {
        let path = stats_path(&self.claw_dir);
        blocking(move || {
            let _guard = lock_guard(&STATS_LOCK);
            if keep_days == 0 || !path.exists() {
                return Ok(0);
            }
            let cutoff = chrono::Local::now().timestamp() - (keep_days as i64 * 86400);

            // Stream through the file line by line, keeping only recent records
            let file = std::fs::File::open(&path)?;
            use std::io::{BufRead, BufReader, Write};
            let reader = BufReader::new(&file);
            let mut kept: Vec<String> = Vec::new();
            let mut removed = 0usize;
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                if let Some(ts) = extract_timestamp(&line)
                    && ts < cutoff
                {
                    removed += 1;
                    continue;
                }
                kept.push(line);
            }

            if removed == 0 {
                tracing::debug!("prune stats: 无需清理");
                return Ok(0);
            }

            let temp_path = path.with_extension("jsonl.tmp");
            {
                let mut file = std::fs::File::create(&temp_path)?;
                for line in &kept {
                    writeln!(file, "{}", line)?;
                }
            }
            std::fs::rename(&temp_path, &path)?;
            tracing::info!("清理了 {} 条过期统计记录", removed);
            Ok(removed)
        })
        .await
    }
}

fn read_range_sync(
    path: &Path,
    from: Option<i64>,
    to: Option<i64>,
) -> anyhow::Result<Vec<crate::stats::TokenRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = std::fs::File::open(path)?;
    use std::io::{BufRead, BufReader};
    let reader = BufReader::new(&file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
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
        match serde_json::from_str::<crate::stats::TokenRecord>(&line) {
            Ok(record) => records.push(record),
            Err(e) => {
                tracing::warn!("跳过损坏的 token 记录行: {}", e);
            }
        }
    }
    Ok(records)
}

// ── SkillRepo ──

#[derive(Clone)]
pub struct FileSkillStore {
    claw_dir: PathBuf,
}

impl FileSkillStore {
    pub(crate) fn new(claw_dir: PathBuf) -> Self {
        Self { claw_dir }
    }
}

#[async_trait]
impl SkillRepo for FileSkillStore {
    async fn list(&self, agent_id: &str) -> anyhow::Result<Vec<SkillEntry>> {
        let dir = skills_dir(&self.claw_dir, agent_id);
        blocking(move || {
            let dir_entries = match std::fs::read_dir(&dir) {
                Ok(d) => d,
                Err(_) => return Ok(Vec::new()),
            };
            let mut entries: Vec<SkillEntry> = dir_entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().map(|ext| ext == "md").unwrap_or(false)
                        && e.path().is_file()
                })
                .filter_map(|e| {
                    let name = e
                        .path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())?;
                    let content = std::fs::read_to_string(e.path()).ok()?;
                    Some(SkillEntry { name, content })
                })
                .collect();
            entries.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(entries)
        })
        .await
    }

    async fn get(
        &self,
        agent_id: &str,
        name: &str,
    ) -> anyhow::Result<Option<crate::skill_store::SkillDefinition>> {
        let dir = skills_dir(&self.claw_dir, agent_id);
        let name = name.to_string();
        blocking(move || {
            let path = dir.join(format!("{}.md", &name));
            if !path.exists() {
                return Ok(None);
            }
            let raw = std::fs::read_to_string(path)?;
            Ok(Some(crate::skill_store::build_skill_definition(
                &name, &raw,
            )))
        })
        .await
    }

    async fn install(&self, agent_id: &str, name: &str, content: &str) -> anyhow::Result<()> {
        let dir = skills_dir(&self.claw_dir, agent_id);
        let name = name.to_string();
        let content = content.to_string();
        blocking(move || {
            std::fs::create_dir_all(&dir)?;
            let path = dir.join(format!("{}.md", &name));
            atomic_write(&path, &content).map_err(anyhow::Error::from)
        })
        .await
    }

    async fn remove(&self, agent_id: &str, name: &str) -> anyhow::Result<()> {
        let dir = skills_dir(&self.claw_dir, agent_id);
        let name = name.to_string();
        blocking(move || {
            let path = dir.join(format!("{}.md", &name));
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            Ok(())
        })
        .await
    }

    async fn list_executable(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<Vec<crate::skill_store::SkillDefinition>> {
        let dir = skills_dir(&self.claw_dir, agent_id);
        blocking(move || {
            let dir_entries = match std::fs::read_dir(&dir) {
                Ok(d) => d,
                Err(_) => return Ok(Vec::new()),
            };
            let mut skills: Vec<crate::skill_store::SkillDefinition> = dir_entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().map(|ext| ext == "md").unwrap_or(false)
                        && e.path().is_file()
                })
                .filter_map(|e| {
                    let path = e.path();
                    let name = path.file_stem().and_then(|s| s.to_str())?;
                    let raw = std::fs::read_to_string(&path).ok()?;
                    let def = crate::skill_store::build_skill_definition(name, &raw);
                    if def.parameters.is_some() {
                        Some(def)
                    } else {
                        None
                    }
                })
                .collect();
            skills.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(skills)
        })
        .await
    }
}

// ── ToolCacheRepo ──

#[derive(Clone)]
pub struct FileToolCacheStore {
    claw_dir: PathBuf,
}

impl FileToolCacheStore {
    pub(crate) fn new(claw_dir: PathBuf) -> Self {
        Self { claw_dir }
    }
}

#[async_trait]
impl ToolCacheRepo for FileToolCacheStore {
    async fn load(&self, agent_id: &str) -> anyhow::Result<HashMap<String, String>> {
        let path = tool_cache_path(&self.claw_dir, agent_id);
        blocking(move || {
            if !path.exists() {
                return Ok(HashMap::new());
            }
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        })
        .await
    }

    async fn save(&self, agent_id: &str, docs: &HashMap<String, String>) -> anyhow::Result<()> {
        let path = tool_cache_path(&self.claw_dir, agent_id);
        let content = serde_json::to_string(docs)?;
        blocking(move || {
            ensure_dir(&path)?;
            atomic_write(&path, &content).map_err(anyhow::Error::from)
        })
        .await
    }
}

// ═══════════════════════════════════════════════════════════════════
//  FileMessageLog — append-only JSONL message log
// ═══════════════════════════════════════════════════════════════════

/// Append-only JSONL message log under `{claw_dir}/sessions/{id}.jsonl`.
///
/// Each line is a serialized `StoredRecord`. New appends go to the end of
/// the file; there is no in-place rewrite — this is the structural fix
/// for the silent-drop bug.
#[derive(Clone)]
pub struct FileMessageLog {
    claw_dir: PathBuf,
}

impl FileMessageLog {
    pub(crate) fn new(claw_dir: PathBuf) -> Self {
        Self { claw_dir }
    }
}

#[async_trait]
impl MessageLog for FileMessageLog {
    async fn append_batch(
        &self,
        session_id: &str,
        messages: &[crate::app::Message],
    ) -> anyhow::Result<()> {
        if messages.is_empty() {
            return Ok(());
        }
        let path = messages_path(&self.claw_dir, session_id);
        let lines: Vec<String> = messages
            .iter()
            .map(crate::message::StoredRecord::from_message)
            .map(|r| r.and_then(|rec| serde_json::to_string(&rec).map_err(Into::into)))
            .collect::<anyhow::Result<Vec<_>>>()?;
        let payload = lines.join("\n") + "\n";

        blocking(move || {
            let _guard = lock_guard(&MESSAGES_LOCK);
            ensure_dir(&path)?;
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            file.write_all(payload.as_bytes())?;
            Ok(())
        })
        .await
    }

    async fn load(
        &self,
        session_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<crate::app::Message>> {
        let path = messages_path(&self.claw_dir, session_id);
        blocking(move || {
            if !path.exists() {
                return Ok(Vec::new());
            }
            let content = std::fs::read_to_string(&path)?;
            let all: Vec<crate::app::Message> = content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .filter_map(|line| {
                    serde_json::from_str::<crate::message::StoredRecord>(line)
                        .ok()?
                        .to_message()
                })
                .collect();
            let limit = limit.min(all.len());
            if limit < all.len() {
                Ok(all[all.len() - limit..].to_vec())
            } else {
                Ok(all)
            }
        })
        .await
    }

    async fn search(&self, query: &str, max_results: usize) -> anyhow::Result<Vec<SearchResult>> {
        let q = query.trim().to_lowercase();
        let claw_dir = self.claw_dir.clone();
        let sessions_dir = sessions_dir(&self.claw_dir);

        blocking(move || {
            if q.is_empty() {
                return Ok(Vec::new());
            }
            let index_path = claw_dir.join("index.json");
            // NOTE: search reads index.json directly — coupled to FileSessionStore format.
            let sessions: Vec<crate::session::SessionMeta> = if index_path.exists() {
                match std::fs::read_to_string(&index_path) {
                    Ok(content) => match serde_json::from_str(&content) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("search: index.json 解析失败: {}", e);
                            Vec::new()
                        }
                    },
                    Err(e) => {
                        tracing::error!("search: 无法读取 index.json: {}", e);
                        Vec::new()
                    }
                }
            } else {
                Vec::new()
            };

            let mut results: Vec<SearchResult> = Vec::new();
            for meta in &sessions {
                let path = sessions_dir.join(format!("{}.jsonl", meta.id));
                if !path.exists() {
                    continue;
                }
                let file = match std::fs::File::open(&path) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                use std::io::Read;
                let mut content = String::new();
                if std::io::BufReader::new(file).read_to_string(&mut content).is_err() {
                    continue;
                }
                let records: Vec<serde_json::Value> = content
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .filter_map(|l| {
                        let rec: crate::message::StoredRecord = serde_json::from_str(l).ok()?;
                        Some(rec.payload)
                    })
                    .collect();

                if super::scan_records_for_query(&records, &q, meta, max_results, &mut results) {
                    return Ok(results);
                }
            }
            Ok(results)
        })
        .await
    }

    async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        let path = messages_path(&self.claw_dir, session_id);
        blocking(move || {
            let _ = std::fs::remove_file(&path);
            Ok(())
        })
        .await
    }

    async fn count(&self, session_id: &str) -> anyhow::Result<usize> {
        let path = messages_path(&self.claw_dir, session_id);
        blocking(move || {
            if !path.exists() {
                return Ok(0);
            }
            let content = std::fs::read_to_string(&path)?;
            Ok(content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .count())
        })
        .await
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Convenience: build ClawStorage from a file directory
// ═══════════════════════════════════════════════════════════════════

impl ClawStorage {
    /// Build with the file-based backend.
    pub fn file(claw_dir: PathBuf) -> Self {
        Self {
            sessions: Box::new(FileSessionStore::new(claw_dir.clone())),
            message_log: std::sync::Arc::new(FileMessageLog::new(claw_dir.clone())),
            api_cache: Box::new(FileApiCacheStore::new(claw_dir.clone())),
            plan_steps: Box::new(FilePlanStepsStore::new(claw_dir.clone())),
            memory: Box::new(FileMemoryStore::new(claw_dir.clone())),
            stats: Box::new(FileStatsStore::new(claw_dir.clone())),
            skills: Box::new(FileSkillStore::new(claw_dir.clone())),
            tool_cache: Box::new(FileToolCacheStore::new(claw_dir)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir() -> PathBuf {
        let id = uuid::Uuid::new_v4().to_string();
        let dir = std::env::temp_dir().join(format!("i-rs-claw-storage-test-{}", id));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    fn test_claw_dir() -> (PathBuf, PathBuf) {
        let dir = test_dir();
        // Return both the temp root and the claw_dir we'll use
        let claw_dir = dir.join("claw");
        (dir, claw_dir)
    }

    // ── SessionRepo ──

    #[tokio::test]
    async fn test_session_load_empty() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileSessionStore::new(claw_dir);
        let sessions = store.load_all().await.unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_session_save_and_load() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileSessionStore::new(claw_dir);
        let sessions = vec![crate::session::SessionMeta {
            id: "test-1".to_string(),
            title: "Hello".to_string(),
            agent_id: "default".to_string(),
            state: crate::session::SessionState::Active,
            created_at: 1000,
            updated_at: 2000,
            message_count: 0,
        }];
        store.save_all(&sessions).await.unwrap();
        let loaded = store.load_all().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "test-1");
        assert_eq!(loaded[0].title, "Hello");
    }

    // ── MessageLog (append-only) ──

    use crate::app::Message;
    use crate::storage::MessageLog;

    #[tokio::test]
    async fn test_message_log_append_and_load() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        log.append_batch(
            "sid",
            &[
                Message::User {
                    text: "hello".into(),
                },
                Message::Assistant {
                    text: "hi".into(),
                    reasoning: String::new(),
                    token_usage: None,
                },
            ],
        )
        .await
        .unwrap();

        let loaded = log.load("sid", usize::MAX).await.unwrap();
        assert_eq!(loaded.len(), 2);
        match &loaded[0] {
            Message::User { text } => assert_eq!(text, "hello"),
            other => panic!("expected User, got {:?}", other),
        }
        match &loaded[1] {
            Message::Assistant { text, .. } => assert_eq!(text, "hi"),
            other => panic!("expected Assistant, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_message_log_preserves_tool_call_step() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        log.append_batch(
            "sid",
            &[Message::ToolCall {
                name: "weight".into(),
                args: "{}".into(),
                result: "ok".into(),
                step: 2,
                total_steps: 5,
            }],
        )
        .await
        .unwrap();

        let loaded = log.load("sid", usize::MAX).await.unwrap();
        match &loaded[0] {
            Message::ToolCall {
                step, total_steps, ..
            } => {
                assert_eq!(*step, 2);
                assert_eq!(*total_steps, 5);
            }
            other => panic!("expected ToolCall, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_message_log_append_batch_is_appending_not_overwriting() {
        // Regression: this is the exact bug class we're fixing. Two
        // successive append_batch calls must accumulate, not clobber.
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        log.append_batch(
            "sid",
            &[Message::User {
                text: "first".into(),
            }],
        )
        .await
        .unwrap();
        log.append_batch(
            "sid",
            &[Message::User {
                text: "second".into(),
            }],
        )
        .await
        .unwrap();

        let loaded = log.load("sid", usize::MAX).await.unwrap();
        assert_eq!(loaded.len(), 2, "both appends must persist");
    }

    #[tokio::test]
    async fn test_message_log_load_limit_returns_tail() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        let msgs: Vec<Message> = (0..5)
            .map(|i| Message::User {
                text: format!("msg{}", i),
            })
            .collect();
        log.append_batch("sid", &msgs).await.unwrap();
        let loaded = log.load("sid", 3).await.unwrap();
        assert_eq!(loaded.len(), 3);
        match &loaded[0] {
            Message::User { text } => assert_eq!(text, "msg2"),
            _ => panic!(),
        }
        match &loaded[2] {
            Message::User { text } => assert_eq!(text, "msg4"),
            _ => panic!(),
        }
    }

    #[tokio::test]
    async fn test_message_log_delete_session() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        log.append_batch("sid", &[Message::User { text: "hi".into() }])
            .await
            .unwrap();
        log.delete_session("sid").await.unwrap();
        assert_eq!(log.load("sid", usize::MAX).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_message_log_count() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        log.append_batch(
            "sid",
            &[
                Message::User { text: "a".into() },
                Message::User { text: "b".into() },
            ],
        )
        .await
        .unwrap();
        assert_eq!(log.count("sid").await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_message_log_search() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir.clone());

        // Search requires session metadata, so wire the session first.
        let sessions = FileSessionStore::new(claw_dir.clone());
        sessions
            .save_all(&[crate::session::SessionMeta {
                id: "sid".into(),
                title: "Test".into(),
                agent_id: "default".into(),
                state: crate::session::SessionState::Active,
                created_at: 1000,
                updated_at: 2000,
                message_count: 0,
            }])
            .await
            .unwrap();

        log.append_batch(
            "sid",
            &[
                Message::User {
                    text: "hello world".into(),
                },
                Message::Assistant {
                    text: "hi there".into(),
                    reasoning: String::new(),
                    token_usage: None,
                },
                Message::ToolCall {
                    name: "weight".into(),
                    args: "{}".into(),
                    result: "ok".into(),
                    step: 0,
                    total_steps: 1,
                },
            ],
        )
        .await
        .unwrap();

        let r = log.search("world", 10).await.unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].session_id, "sid");

        let r = log.search("weight", 10).await.unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].message_type, "tool_call");

        let r = log.search("nonexistent", 10).await.unwrap();
        assert!(r.is_empty());
    }

    // ── ApiCacheRepo ──

    #[tokio::test]
    async fn test_api_cache_save_load() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileApiCacheStore::new(claw_dir);
        let msgs = vec![serde_json::json!({"role": "user", "content": "hello"})];
        store.save("sid", &msgs).await.unwrap();
        let loaded = store.load("sid").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_api_cache_load_empty() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileApiCacheStore::new(claw_dir);
        let loaded = store.load("nonexistent").await.unwrap();
        assert!(loaded.is_none());
    }

    // ── PlanStepsRepo ──

    #[tokio::test]
    async fn test_plan_steps_save_load() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FilePlanStepsStore::new(claw_dir);
        let steps = vec![
            crate::app::PlanStep {
                description: "Step 1".to_string(),
                done: false,
            },
            crate::app::PlanStep {
                description: "Step 2".to_string(),
                done: true,
            },
        ];
        store.save("sid", &steps).await.unwrap();
        let loaded = store.load("sid").await.unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(!loaded[0].done);
        assert!(loaded[1].done);
    }

    // ── MemoryRepo ──

    #[tokio::test]
    async fn test_memory_load_default() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileMemoryStore::new(claw_dir);
        let mem = store.load("agent1").await.unwrap();
        assert!(mem.is_none(), "no memory file yet → None");
    }

    #[tokio::test]
    async fn test_memory_save_and_load() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileMemoryStore::new(claw_dir);
        let mut mem = store
            .load("agent1")
            .await
            .unwrap()
            .unwrap_or_else(crate::memory::CrossSessionMemory::default_memory);
        mem.set_user_name("Alice");
        store.save("agent1", &mem).await.unwrap();

        let loaded = store
            .load("agent1")
            .await
            .unwrap()
            .expect("should exist after save");
        assert!(loaded.has_user_profile());
        let formatted = loaded.format_user_memory();
        assert!(formatted.contains("Alice"));
    }

    // ── StatsRepo ──

    #[tokio::test]
    async fn test_stats_append_and_read() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileStatsStore::new(claw_dir);
        let record = crate::stats::TokenRecord {
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
        store.upsert_batch(&[record]).await.unwrap();

        let records = store.read_range(None, None).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "test-1");

        let records = store.read_range(Some(1716220801), None).await.unwrap();
        assert_eq!(records.len(), 0);
    }

    // ── SkillRepo ──

    #[tokio::test]
    async fn test_skill_install_and_list() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileSkillStore::new(claw_dir);
        store
            .install("agent1", "my-skill", "test content")
            .await
            .unwrap();
        let skills = store.list("agent1").await.unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "my-skill");
    }

    #[tokio::test]
    async fn test_skill_get_and_remove() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileSkillStore::new(claw_dir);
        store
            .install("agent1", "my-skill", "content here")
            .await
            .unwrap();
        let skill = store.get("agent1", "my-skill").await.unwrap();
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().name, "my-skill");

        store.remove("agent1", "my-skill").await.unwrap();
        assert!(store.get("agent1", "my-skill").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_skill_executable_filtering() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileSkillStore::new(claw_dir);
        store
            .install(
                "agent1",
                "exec-skill",
                r#"---
description = "test"
[parameters]
type = "object"
---
body"#,
            )
            .await
            .unwrap();
        store
            .install("agent1", "plain-skill", "just text")
            .await
            .unwrap();

        let execs = store.list_executable("agent1").await.unwrap();
        assert_eq!(execs.len(), 1);
        assert_eq!(execs[0].name, "exec-skill");
    }

    // ── ToolCacheRepo ──

    #[tokio::test]
    async fn test_tool_cache_save_load() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileToolCacheStore::new(claw_dir);
        let mut docs = HashMap::new();
        docs.insert("weight".to_string(), "tool doc content".to_string());
        store.save("agent1", &docs).await.unwrap();

        let loaded = store.load("agent1").await.unwrap();
        assert_eq!(loaded.get("weight").unwrap(), "tool doc content");
    }

    #[tokio::test]
    async fn test_tool_cache_load_empty() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileToolCacheStore::new(claw_dir);
        let loaded = store.load("nonexistent").await.unwrap();
        assert!(loaded.is_empty());
    }

    // ── ClawStorage integration ──

    #[tokio::test]
    async fn test_claw_storage_file() {
        let (_root, claw_dir) = test_claw_dir();
        let storage = ClawStorage::file(claw_dir);

        storage
            .sessions
            .save_all(&[crate::session::SessionMeta {
                id: "s1".to_string(),
                title: "Test".to_string(),
                agent_id: "default".to_string(),
                state: crate::session::SessionState::Active,
                created_at: 1000,
                updated_at: 2000,
                message_count: 0,
            }])
            .await
            .unwrap();

        let mem = storage
            .memory
            .load("default")
            .await
            .unwrap()
            .unwrap_or_else(crate::memory::CrossSessionMemory::default_memory);
        assert!(!mem.has_user_profile());
    }
}

/// Sync I/O primitives for the file SessionRepo backend.
mod session_io {
    use std::path::Path;

    use super::*;

    pub fn load_sessions_sync(claw_dir: &Path) -> anyhow::Result<Vec<crate::session::SessionMeta>> {
        let path = index_path(claw_dir);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content)
            .inspect_err(|e| tracing::error!("index.json 损坏: {} — 不会静默清空", e))
            .map_err(|e| anyhow::anyhow!("index.json 损坏: {}", e))
    }

    pub fn save_sessions_sync(
        claw_dir: &Path,
        sessions: &[crate::session::SessionMeta],
    ) -> anyhow::Result<()> {
        let path = index_path(claw_dir);
        let content = serde_json::to_string_pretty(sessions)?;
        ensure_dir(&path)?;
        if path.exists() {
            let bak = path.with_extension("json.bak");
            std::fs::copy(&path, &bak).ok();
        }
        atomic_write(&path, &content).map_err(anyhow::Error::from)
    }
}
