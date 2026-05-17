/// Lightweight semantic search for conversation history.
///
/// Provides TF-IDF-like relevance ranking over message text, with optional
/// remote embedding API support for deeper semantic matching.
///
/// Usage:
///   let searcher = SemanticSearch::new(&claw_dir);
///   let results = searcher.search("running weight last week", 5)?;

use crate::convstore::{ConvStore, SearchResult};
use std::collections::{BTreeMap, HashSet};

/// A search result with a relevance score.
#[derive(Debug, Clone)]
pub struct ScoredResult {
    pub result: SearchResult,
    pub score: f64,
}

/// Lightweight semantic search engine.
pub struct SemanticSearch {
    conv_store: ConvStore,
}

impl SemanticSearch {
    pub fn new(claw_dir: std::path::PathBuf) -> Self {
        Self {
            conv_store: ConvStore::new(claw_dir),
        }
    }

    /// Search conversations with TF-IDF relevance scoring.
    ///
    /// 1. First uses ConvStore's keyword search to find matching messages.
    /// 2. Then scores results by TF-IDF relevance against the query.
    /// 3. Returns top results sorted by score descending.
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
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Remove duplicates (same session_id + same excerpt)
        let mut seen = HashSet::new();
        scored.retain(|s| {
            let key = format!("{}:{}", s.result.session_id, s.result.excerpt);
            seen.insert(key)
        });

        scored.truncate(max_results);
        scored
    }
}

/// Tokenize text into lowercase words (English + Chinese).
fn tokenize(text: &str) -> Vec<String> {
    let text = text.to_lowercase();
    let mut tokens = Vec::new();

    // Split on whitespace and punctuation
    let mut current = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() || c.is_ascii() || c > '\x7f' {
            // Include CJK characters as individual tokens
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
fn term_frequency(tokens: &[String]) -> BTreeMap<String, usize> {
    let mut tf = BTreeMap::new();
    for token in tokens {
        *tf.entry(token.clone()).or_insert(0) += 1;
    }
    tf
}

/// Compute TF-IDF relevance score between a search result and query.
///
/// Score is based on:
/// - Term frequency: how often query terms appear in the result text
/// - Inverse document frequency: rare terms get higher weight (simulated)
/// - Length normalization: shorter matches get slight boost
/// - Bonus for title matches (session title containing query terms)
fn compute_relevance(
    result: &SearchResult,
    query_terms: &[String],
    query_tf: &BTreeMap<String, usize>,
) -> f64 {
    if query_terms.is_empty() {
        return 0.0;
    }

    // Build result text for scoring
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

        // Term frequency in result
        let tf = *result_tf.get(term).unwrap_or(&0) as f64;

        // Simulated IDF: rarer terms get higher weight
        // Common Chinese stop words get lower weight
        let idf = if is_stop_word(term) {
            0.3
        } else if term.len() <= 1 {
            0.5
        } else {
            1.0
        };

        // TF-IDF contribution
        score += query_weight * tf * idf / total_terms;
    }

    // Title match bonus
    let title_lower = result.session_title.to_lowercase();
    for term in query_terms {
        if title_lower.contains(term) {
            score += 0.5;
        }
    }

    // Length normalization: slightly boost shorter, denser matches
    let len_norm = (1.0 + 10.0 / total_terms).min(2.0);
    score *= len_norm;

    score
}

/// Common stop words with low semantic value.
fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "的" | "了" | "在" | "是" | "我" | "有" | "和" | "就"
            | "不" | "人" | "都" | "一" | "个" | "上" | "也"
            | "很" | "到" | "说" | "要" | "去" | "你" | "会"
            | "着" | "没有" | "看" | "好" | "自己" | "这"
            | "the" | "a" | "an" | "is" | "are" | "was" | "were"
            | "be" | "been" | "being" | "have" | "has" | "had"
            | "do" | "does" | "did" | "will" | "would" | "can"
            | "could" | "may" | "might" | "shall" | "should"
            | "to" | "of" | "in" | "for" | "on" | "with" | "at"
            | "by" | "from" | "and" | "or" | "but" | "not" | "no"
            | "this" | "that" | "it" | "its" | "i" | "you" | "he"
            | "she" | "we" | "they" | "me" | "him" | "her" | "us"
            | "them"
    )
}
