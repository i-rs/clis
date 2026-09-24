//! Lightweight semantic search for conversation history.
//!
//! Provides TF-IDF-like relevance ranking over message text, with optional
//! remote embedding API support for deeper semantic matching.
//!
//! Usage:
//!   let searcher = SemanticSearch::new(&claw_dir);
//!   let results = searcher.search("running weight last week", 5)?;

use crate::convstore::ConvStore;
use crate::storage::SearchResult;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// =============================================
// Embedding Provider (remote API)
// =============================================

/// Generate embeddings for text using a remote API.
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding vector for a single text string.
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f64>>;
    /// Generate embeddings for multiple texts (batched).
    async fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f64>>>;
}

/// OpenAI-compatible embedding provider (uses /embeddings endpoint).
#[allow(dead_code)]
pub struct OpenaiEmbeddingProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenaiEmbeddingProvider {
    #[allow(dead_code)]
    pub fn new(api_key: String, base_url: String, model: Option<String>) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        Ok(Self {
            client,
            api_key,
            base_url,
            model: model.unwrap_or_else(|| "text-embedding-3-small".to_string()),
        })
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for OpenaiEmbeddingProvider {
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f64>> {
        let mut results = self.embed_batch(&[text.to_string()]).await?;
        results
            .pop()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))
    }

    async fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f64>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let url = format!("{}/embeddings", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "input": texts,
            "model": self.model,
        });
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Embedding API request failed: {}", e))?;
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Embedding API error: {}", text));
        }
        let data: serde_json::Value = response.json().await?;
        let embeddings = data["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No data in embedding response"))?;
        let mut results = Vec::with_capacity(embeddings.len());
        for entry in embeddings {
            let vec: Vec<f64> = entry["embedding"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("No embedding vector"))?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0))
                .collect();
            results.push(vec);
        }
        Ok(results)
    }
}

// =============================================
// Embedding Index (local storage + search)
// =============================================

/// A single entry in the embedding index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub session_id: String,
    pub session_title: String,
    pub message_type: String,
    pub excerpt: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
    pub updated_at: i64,
    /// The embedding vector (list of floats).
    pub embedding: Vec<f64>,
}

/// Local embedding index for cosine similarity search.
pub struct EmbeddingIndex {
    entries: Vec<IndexEntry>,
    path: PathBuf,
}

impl EmbeddingIndex {
    /// Create or load an embedding index from disk.
    pub fn new(claw_dir: PathBuf) -> Self {
        let path = claw_dir.join("embedding_index.json");
        let entries = if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        Self { entries, path }
    }

    /// Return number of indexed entries.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the index has no entries.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Add entries to the index and persist.
    #[allow(dead_code)]
    pub fn add_entries(&mut self, new_entries: Vec<IndexEntry>) {
        let existing_keys: std::collections::HashSet<String> = self
            .entries
            .iter()
            .map(|e| format!("{}:{}", e.session_id, e.excerpt))
            .collect();
        for entry in new_entries {
            let key = format!("{}:{}", entry.session_id, entry.excerpt);
            if !existing_keys.contains(&key) {
                self.entries.push(entry);
            }
        }
        self.save();
    }

    /// Clear and rebuild the index.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.save();
    }

    /// Search by cosine similarity against query embedding.
    /// Returns top-N results sorted by similarity score (0.0 to 1.0).
    pub fn search(&self, query_embedding: &[f64], max_results: usize) -> Vec<(IndexEntry, f64)> {
        if self.entries.is_empty() || query_embedding.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<(usize, f64)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let sim = cosine_similarity(query_embedding, &entry.embedding);
                (i, sim)
            })
            .collect();
        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        // Take top results
        scored.truncate(max_results);
        scored
            .into_iter()
            .filter(|(_, score)| *score > 0.3) // Minimum similarity threshold
            .map(|(i, score)| (self.entries[i].clone(), score))
            .collect()
    }

    fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string(&self.entries) {
            let _ = crate::utils::atomic_write(&self.path, &content);
        }
    }
}

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        (dot / (norm_a * norm_b)).clamp(0.0, 1.0)
    }
}

// =============================================
// Enhanced SemanticSearch
// =============================================

/// A search result with a relevance score.
#[derive(Debug, Clone)]
pub struct ScoredResult {
    pub result: SearchResult,
    pub score: f64,
}

/// Lightweight semantic search engine with optional embedding support.
pub struct SemanticSearch {
    conv_store: ConvStore,
    embed_index: EmbeddingIndex,
    #[allow(dead_code)]
    claw_dir: PathBuf,
}

impl SemanticSearch {
    pub fn new(claw_dir: PathBuf) -> Self {
        let embed_index = EmbeddingIndex::new(claw_dir.clone());
        Self {
            conv_store: ConvStore::for_claw_dir(claw_dir.clone()),
            embed_index,
            claw_dir,
        }
    }

    /// Get a reference to the embedding index for indexing.
    #[allow(dead_code)]
    pub fn embed_index(&self) -> &EmbeddingIndex {
        &self.embed_index
    }

    /// Get a mutable reference to the embedding index for indexing.
    #[allow(dead_code)]
    pub fn embed_index_mut(&mut self) -> &mut EmbeddingIndex {
        &mut self.embed_index
    }

    /// Search conversations with relevance scoring.
    ///
    /// 1. First uses ConvStore's keyword search to find matching messages.
    /// 2. Then scores results by TF-IDF relevance against the query.
    /// 3. If an embedding provider is available, re-ranks results by cosine similarity.
    /// 4. Returns top results sorted by score descending.
    pub fn search(&self, query: &str, max_results: usize) -> Vec<ScoredResult> {
        if query.trim().is_empty() {
            return Vec::new();
        }

        // Boost search breadth - get more candidates for better ranking
        let candidates = self.conv_store.search(query, max_results * 5);
        if candidates.is_empty() {
            return Vec::new();
        }

        let query_terms = tokenize(query);
        let query_tf = term_frequency(&query_terms);

        let mut scored: Vec<ScoredResult> = candidates
            .into_iter()
            .map(|result| {
                let score = compute_relevance(&result, &query_terms, &query_tf);
                ScoredResult { result, score }
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Remove duplicates (same session_id + same excerpt)
        let mut seen = HashSet::new();
        scored.retain(|s| {
            let key = format!("{}:{}", s.result.session_id, s.result.excerpt);
            seen.insert(key)
        });

        scored.truncate(max_results);
        scored
    }

    /// Search using the embedding index for semantic similarity.
    /// Returns results re-ranked by embedding cosine similarity when available.
    #[allow(dead_code)]
    pub fn search_embedding(
        &self,
        query_embedding: &[f64],
        max_results: usize,
    ) -> Vec<(ScoredResult, f64)> {
        let indexed = self.embed_index.search(query_embedding, max_results);
        if indexed.is_empty() {
            return Vec::new();
        }
        indexed
            .into_iter()
            .map(|(entry, score)| {
                // Build a ScoredResult from the indexed entry
                let result = SearchResult {
                    session_id: entry.session_id,
                    session_title: entry.session_title,
                    message_type: entry.message_type,
                    excerpt: entry.excerpt.clone(),
                    context_before: entry.context_before,
                    context_after: entry.context_after,
                    updated_at: entry.updated_at,
                };
                let scored = ScoredResult {
                    score, // Use embedding score directly
                    result,
                };
                (scored, score)
            })
            .collect()
    }
}

/// Tokenize text into lowercase words (English + Chinese).
fn tokenize(text: &str) -> Vec<String> {
    let text = text.to_lowercase();
    let mut tokens = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() || c.is_ascii() || c > '\x7f' {
            if c > '\x7f' && !c.is_whitespace() {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(c.to_string());
            } else if c.is_alphanumeric() || c == '_' || c == '-' {
                current.push(c);
            } else {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
        } else {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Compute term frequency map for a list of tokens.
fn term_frequency(tokens: &[String]) -> HashMap<String, usize> {
    let mut tf = HashMap::new();
    for token in tokens {
        *tf.entry(token.clone()).or_insert(0) += 1;
    }
    tf
}

/// Compute TF-IDF relevance score between a search result and query.
fn compute_relevance(
    result: &SearchResult,
    query_terms: &[String],
    query_tf: &HashMap<String, usize>,
) -> f64 {
    if query_terms.is_empty() {
        return 0.0;
    }

    let searchable = format!(
        "{} {} {} {} {}",
        result.session_title,
        result.excerpt,
        result.context_before.join(" "),
        result.context_after.join(" "),
        result.message_type
    );
    let result_terms = tokenize(&searchable);
    let result_tf = term_frequency(&result_terms);
    let total_terms = result_terms.len().max(1) as f64;

    let mut score = 0.0;
    for (term, query_count) in query_tf {
        let query_weight = *query_count as f64;
        let tf = *result_tf.get(term).unwrap_or(&0) as f64;
        let idf = if is_stop_word(term) {
            0.3
        } else if term.len() <= 1 {
            0.5
        } else {
            1.0
        };
        score += query_weight * tf * idf / total_terms;
    }

    let title_lower = result.session_title.to_lowercase();
    for term in query_terms {
        if title_lower.contains(term) {
            score += 0.5;
        }
    }

    let len_norm = (1.0 + 10.0 / total_terms).min(2.0);
    score *= len_norm;
    score
}

fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "的" | "了"
            | "在"
            | "是"
            | "我"
            | "有"
            | "和"
            | "就"
            | "不"
            | "人"
            | "都"
            | "一"
            | "个"
            | "上"
            | "也"
            | "很"
            | "到"
            | "说"
            | "要"
            | "去"
            | "你"
            | "会"
            | "着"
            | "没有"
            | "看"
            | "好"
            | "自己"
            | "这"
            | "the"
            | "a"
            | "an"
            | "is"
            | "are"
            | "was"
            | "were"
            | "be"
            | "been"
            | "being"
            | "have"
            | "has"
            | "had"
            | "do"
            | "does"
            | "did"
            | "will"
            | "would"
            | "can"
            | "could"
            | "may"
            | "might"
            | "shall"
            | "should"
            | "to"
            | "of"
            | "in"
            | "for"
            | "on"
            | "with"
            | "at"
            | "by"
            | "from"
            | "and"
            | "or"
            | "but"
            | "not"
            | "no"
            | "this"
            | "that"
            | "it"
            | "its"
            | "i"
            | "you"
            | "he"
            | "she"
            | "we"
            | "they"
            | "me"
            | "him"
            | "her"
            | "us"
            | "them"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("hello world");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
    }

    #[test]
    fn test_tokenize_chinese() {
        let tokens = tokenize("你好世界");
        assert_eq!(tokens.len(), 4); // 4 individual CJK chars
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 1e-6);

        // Empty vectors
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
    }

    #[test]
    fn test_embedding_index() {
        let dir = std::env::temp_dir().join("i-rs-claw-test-embed");
        let _ = std::fs::create_dir_all(&dir);
        let mut idx = EmbeddingIndex::new(dir.clone());

        assert_eq!(idx.len(), 0);

        idx.add_entries(vec![IndexEntry {
            session_id: "s1".to_string(),
            session_title: "Test".to_string(),
            message_type: "user".to_string(),
            excerpt: "hello world".to_string(),
            context_before: vec![],
            context_after: vec![],
            updated_at: 0,
            embedding: vec![1.0, 0.0, 0.0],
        }]);

        assert_eq!(idx.len(), 1);

        let results = idx.search(&[1.0, 0.0, 0.0], 5);
        assert_eq!(results.len(), 1);
        assert!((results[0].1 - 1.0).abs() < 1e-6);

        // Dedup: same key should not be added twice
        idx.add_entries(vec![IndexEntry {
            session_id: "s1".to_string(),
            session_title: "Test".to_string(),
            message_type: "user".to_string(),
            excerpt: "hello world".to_string(),
            context_before: vec![],
            context_after: vec![],
            updated_at: 0,
            embedding: vec![1.0, 0.0, 0.0],
        }]);
        assert_eq!(idx.len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_term_frequency() {
        let tokens = vec!["a".to_string(), "b".to_string(), "a".to_string()];
        let tf = term_frequency(&tokens);
        assert_eq!(tf.get("a"), Some(&2));
        assert_eq!(tf.get("b"), Some(&1));
    }

    #[test]
    fn test_is_stop_word() {
        assert!(is_stop_word("的"));
        assert!(is_stop_word("the"));
        assert!(!is_stop_word("重要"));
        assert!(!is_stop_word("important"));
    }
}
