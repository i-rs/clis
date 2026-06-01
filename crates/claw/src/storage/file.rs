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
//!
//! NOTE: Not all stores are yet wired into every consumer — dead_code warnings
//! are expected during progressive rollout.
#![allow(dead_code)]

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::*;

static FILE_WRITE_LOCK: Mutex<()> = Mutex::new(());

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

fn extract_timestamp(line: &str) -> Option<i64> {
    let marker = "\"timestamp\":";
    line.find(marker).and_then(|pos| {
        let rest = &line[pos + marker.len()..];
        let end = rest.find(|c: char| !c.is_ascii_digit() && c != '-')?;
        rest[..end].parse::<i64>().ok()
    })
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
            Ok(serde_json::from_str(&content).unwrap_or_default())
        })
        .await
    }

    async fn save_all(&self, sessions: &[crate::session::SessionMeta]) -> anyhow::Result<()> {
        let path = index_path(&self.claw_dir);
        let content = serde_json::to_string_pretty(sessions)?;
        blocking(move || {
            ensure_dir(&path)?;
            atomic_write(&path, &content).map_err(anyhow::Error::from)
        })
        .await
    }
}

// ── MessageRepo ──

#[derive(Clone)]
pub struct FileMessageStore {
    claw_dir: PathBuf,
}

impl FileMessageStore {
    pub(crate) fn new(claw_dir: PathBuf) -> Self {
        Self { claw_dir }
    }
}

#[async_trait]
impl MessageRepo for FileMessageStore {
    async fn append(&self, session_id: &str, entry: &serde_json::Value) -> anyhow::Result<()> {
        let path = messages_path(&self.claw_dir, session_id);
        let line = serde_json::to_string(entry)?;
        blocking(move || {
            let _guard = FILE_WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            ensure_dir(&path)?;
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            writeln!(file, "{}", line)?;
            Ok(())
        })
        .await
    }

    async fn load(&self, session_id: &str, limit: usize) -> anyhow::Result<Vec<serde_json::Value>> {
        let path = messages_path(&self.claw_dir, session_id);
        blocking(move || {
            if !path.exists() {
                return Ok(Vec::new());
            }
            let content = std::fs::read_to_string(&path)?;
            let all_lines: Vec<serde_json::Value> = content
                .lines()
                .filter_map(|line| {
                    if line.trim().is_empty() {
                        None
                    } else {
                        serde_json::from_str(line).ok()
                    }
                })
                .collect();
            if all_lines.len() > limit {
                Ok(all_lines[all_lines.len() - limit..].to_vec())
            } else {
                Ok(all_lines)
            }
        })
        .await
    }

    async fn save_all(
        &self,
        session_id: &str,
        records: &[serde_json::Value],
    ) -> anyhow::Result<()> {
        let path = messages_path(&self.claw_dir, session_id);
        let content: String = records
            .iter()
            .filter_map(|record| serde_json::to_string(record).ok().map(|line| line + "\n"))
            .collect();
        blocking(move || {
            ensure_dir(&path)?;
            atomic_write(&path, &content).map_err(anyhow::Error::from)
        })
        .await
    }

    async fn search(&self, query: &str, max_results: usize) -> anyhow::Result<Vec<SearchResult>> {
        let query = query.to_lowercase();
        let claw_dir = self.claw_dir.clone();
        let sessions_dir = sessions_dir(&self.claw_dir);

        blocking(move || {
            if query.trim().is_empty() {
                return Ok(Vec::new());
            }

            let index_path = claw_dir.join("index.json");
            let sessions: Vec<crate::session::SessionMeta> = if index_path.exists() {
                std::fs::read_to_string(&index_path)
                    .ok()
                    .and_then(|c| serde_json::from_str(&c).ok())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            let mut results: Vec<SearchResult> = Vec::new();

            for meta in &sessions {
                let jsonl_path = sessions_dir.join(format!("{}.jsonl", meta.id));
                if !jsonl_path.exists() {
                    continue;
                }
                let content = match std::fs::read_to_string(&jsonl_path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let lines: Vec<serde_json::Value> = content
                    .lines()
                    .filter_map(|line| {
                        if line.trim().is_empty() {
                            None
                        } else {
                            serde_json::from_str(line).ok()
                        }
                    })
                    .collect();

                for (i, msg) in lines.iter().enumerate() {
                    let msg_type = msg.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    let text = msg.get("text").and_then(|t| t.as_str()).unwrap_or("");
                    let name = msg.get("name").and_then(|n| n.as_str()).unwrap_or("");

                    let searchable = match msg_type {
                        "user" | "assistant" | "error" => text.to_lowercase(),
                        "tool_call" => format!("[tool: {}]", name).to_lowercase(),
                        _ => continue,
                    };

                    if !searchable.contains(&query) {
                        continue;
                    }

                    let excerpt = match msg_type {
                        "tool_call" => format!("[工具调用: {}]", name),
                        _ => {
                            let t: String = text.chars().take(200).collect();
                            if text.len() > 200 {
                                format!("{}...", t)
                            } else {
                                t
                            }
                        }
                    };

                    let context_before: Vec<String> = lines[i.saturating_sub(2)..i]
                        .iter()
                        .filter_map(|m| {
                            let t = m.get("text").and_then(|v| v.as_str())?;
                            let s: String = t.chars().take(100).collect();
                            Some(s)
                        })
                        .collect();

                    let context_after: Vec<String> = if i + 1 < lines.len() {
                        let end = std::cmp::min(i + 1, lines.len() - 1);
                        lines[i + 1..=end]
                            .iter()
                            .filter_map(|m| {
                                let t = m.get("text").and_then(|v| v.as_str())?;
                                let s: String = t.chars().take(100).collect();
                                Some(s)
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };

                    results.push(SearchResult {
                        session_id: meta.id.clone(),
                        session_title: meta.title.clone(),
                        message_type: msg_type.to_string(),
                        excerpt,
                        context_before,
                        context_after,
                        updated_at: meta.updated_at,
                    });

                    if results.len() >= max_results {
                        return Ok(results);
                    }
                }
            }

            Ok(results)
        })
        .await
    }

    async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        let msg = messages_path(&self.claw_dir, session_id);
        let api = api_cache_path(&self.claw_dir, session_id);
        let plan = plan_steps_path(&self.claw_dir, session_id);
        blocking(move || {
            let _ = std::fs::remove_file(&msg);
            let _ = std::fs::remove_file(&api);
            let _ = std::fs::remove_file(&plan);
            Ok(())
        })
        .await
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
            Ok(serde_json::from_str(&content).ok())
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
            Ok(serde_json::from_str(&content).unwrap_or_default())
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
    async fn load(&self, agent_id: &str) -> anyhow::Result<crate::memory::CrossSessionMemory> {
        let path = memory_path(&self.claw_dir, agent_id);
        blocking(move || Ok(crate::memory::CrossSessionMemory::load_from(&path))).await
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
    async fn append_batch(&self, records: &[crate::stats::TokenRecord]) -> anyhow::Result<()> {
        let path = stats_path(&self.claw_dir);
        let json_lines: Vec<String> = records
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?;

        blocking(move || {
            let _guard = FILE_WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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
            if keep_days == 0 || !path.exists() {
                return Ok(0);
            }
            let cutoff = chrono::Local::now().timestamp() - (keep_days as i64 * 86400);
            let all = read_range_sync(&path, None, None)?;
            let before = all.len();
            let kept: Vec<&crate::stats::TokenRecord> =
                all.iter().filter(|r| r.timestamp >= cutoff).collect();
            let removed = before - kept.len();
            if removed == 0 {
                return Ok(0);
            }
            let temp_path = path.with_extension("jsonl.tmp");
            {
                use std::io::Write;
                let mut file = std::fs::File::create(&temp_path)?;
                for record in &kept {
                    let json = serde_json::to_string(record)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                    writeln!(file, "{}", json)?;
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
            std::fs::write(&path, &content)?;
            Ok(())
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

    async fn format_skills(&self, agent_id: &str) -> anyhow::Result<String> {
        let dir = skills_dir(&self.claw_dir, agent_id);
        blocking(move || {
            let dir_entries = match std::fs::read_dir(&dir) {
                Ok(d) => d,
                Err(_) => return Ok(String::new()),
            };
            let mut entries: Vec<_> = dir_entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().map(|ext| ext == "md").unwrap_or(false)
                        && e.path().is_file()
                })
                .collect();
            entries.sort_by_key(|e| e.file_name());

            if entries.is_empty() {
                return Ok(String::new());
            }

            let mut result = String::from("## 用户技能\n\n");
            result.push_str("以下是用户定义的自定义技能指令，请在对话中遵循这些指导：\n");

            for entry in &entries {
                let path = entry.path();
                let skill_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                if let Ok(raw) = std::fs::read_to_string(&path) {
                    let trimmed = raw.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let (frontmatter, body) = crate::skill_store::parse_frontmatter(trimmed);
                    let heading = frontmatter
                        .as_ref()
                        .and_then(|t| t.get("description"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(skill_name);
                    let display_content = if body.is_empty() { trimmed } else { body };
                    result.push_str(&format!("\n### {}\n{}\n", heading, display_content));
                }
            }

            Ok(result)
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
            Ok(serde_json::from_str(&content).unwrap_or_default())
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
//  Convenience: build ClawStorage from a file directory
// ═══════════════════════════════════════════════════════════════════

impl ClawStorage {
    /// Build with the file-based backend.
    pub fn file(claw_dir: PathBuf) -> Self {
        Self {
            sessions: Box::new(FileSessionStore::new(claw_dir.clone())),
            messages: Box::new(FileMessageStore::new(claw_dir.clone())),
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

    // ── MessageRepo ──

    #[tokio::test]
    async fn test_message_append_and_load() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileMessageStore::new(claw_dir);
        let msg = serde_json::json!({"type": "user", "text": "hello"});
        store.append("sid", &msg).await.unwrap();
        store
            .append(
                "sid",
                &serde_json::json!({"type": "assistant", "text": "hi"}),
            )
            .await
            .unwrap();

        let loaded = store.load("sid", 10).await.unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0]["text"], "hello");
        assert_eq!(loaded[1]["text"], "hi");
    }

    #[tokio::test]
    async fn test_message_load_limit() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileMessageStore::new(claw_dir);
        for i in 0..5 {
            store
                .append("sid", &serde_json::json!({"n": i}))
                .await
                .unwrap();
        }
        let loaded = store.load("sid", 3).await.unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0]["n"], 2);
        assert_eq!(loaded[2]["n"], 4);
    }

    #[tokio::test]
    async fn test_message_save_all() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileMessageStore::new(claw_dir);
        let msgs = vec![
            serde_json::json!({"type": "user", "text": "a"}),
            serde_json::json!({"type": "assistant", "text": "b"}),
        ];
        store.save_all("sid", &msgs).await.unwrap();
        store.save_all("sid", &msgs).await.unwrap(); // overwrite
        let loaded = store.load("sid", 10).await.unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[tokio::test]
    async fn test_message_search() {
        let (_root, claw_dir) = test_claw_dir();
        let sess_store = FileSessionStore::new(claw_dir.clone());
        let msg_store = FileMessageStore::new(claw_dir);

        sess_store
            .save_all(&[crate::session::SessionMeta {
                id: "sid".to_string(),
                title: "Test Session".to_string(),
                agent_id: "default".to_string(),
                state: crate::session::SessionState::Active,
                created_at: 1000,
                updated_at: 2000,
                message_count: 0,
            }])
            .await
            .unwrap();

        msg_store
            .append(
                "sid",
                &serde_json::json!({"type": "user", "text": "Hello world"}),
            )
            .await
            .unwrap();
        msg_store
            .append(
                "sid",
                &serde_json::json!({"type": "assistant", "text": "Hi there"}),
            )
            .await
            .unwrap();
        msg_store
            .append(
                "sid",
                &serde_json::json!({"type": "tool_call", "name": "weight", "text": ""}),
            )
            .await
            .unwrap();

        let results = msg_store.search("world", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, "sid");

        let results = msg_store.search("weight", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message_type, "tool_call");

        let results = msg_store.search("nonexistent", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_message_delete_session() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileMessageStore::new(claw_dir);
        store
            .append("sid", &serde_json::json!({"type": "user", "text": "hi"}))
            .await
            .unwrap();
        store.delete_session("sid").await.unwrap();
        let loaded = store.load("sid", 10).await.unwrap();
        assert!(loaded.is_empty());
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
        assert!(!mem.has_user_profile());
    }

    #[tokio::test]
    async fn test_memory_save_and_load() {
        let (_root, claw_dir) = test_claw_dir();
        let store = FileMemoryStore::new(claw_dir);
        let mut mem = store.load("agent1").await.unwrap();
        mem.set_user_name("Alice");
        store.save("agent1", &mem).await.unwrap();

        let loaded = store.load("agent1").await.unwrap();
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
        store.append_batch(&[record]).await.unwrap();

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

        storage
            .messages
            .append("s1", &serde_json::json!({"type": "user", "text": "hi"}))
            .await
            .unwrap();
        let msgs = storage.messages.load("s1", 10).await.unwrap();
        assert_eq!(msgs.len(), 1);

        let mem = storage.memory.load("default").await.unwrap();
        assert!(!mem.has_user_profile());
    }
}
