use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagDocument {
    pub id: String,
    pub content: String,
    pub source: String,
    pub metadata: std::collections::HashMap<String, String>,
    pub chunk_index: usize,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQuery {
    pub query: String,
    pub top_k: usize,
    pub min_score: f64,
    pub source_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResult {
    pub query: String,
    pub documents: Vec<ScoredDocument>,
    pub answer_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredDocument {
    pub document: RagDocument,
    pub score: f64,
}

pub struct ChunkConfig {
    pub chunk_size: usize,
    pub overlap: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 500,
            overlap: 50,
        }
    }
}

pub fn chunk_text(text: &str, config: &ChunkConfig) -> Vec<String> {
    if text.len() <= config.chunk_size {
        return if text.is_empty() { vec![] } else { vec![text.to_string()] };
    }
    let mut chunks = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut start = 0;
    while start < chars.len() {
        let end = (start + config.chunk_size).min(chars.len());
        let chunk: String = chars[start..end].iter().collect();
        chunks.push(chunk);
        start += config.chunk_size - config.overlap;
        if start >= chars.len() {
            break;
        }
    }
    chunks
}

pub struct RagPipeline {
    documents: Vec<RagDocument>,
    chunk_config: ChunkConfig,
}

impl RagPipeline {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            chunk_config: ChunkConfig::default(),
        }
    }

    pub fn with_chunk_config(mut self, config: ChunkConfig) -> Self {
        self.chunk_config = config;
        self
    }

    pub fn ingest(&mut self, content: &str, source: &str) -> Vec<String> {
        let chunks = chunk_text(content, &self.chunk_config);
        let now = chrono::Utc::now().timestamp();
        let mut ids = Vec::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let id = format!("{}_{}", source, i);
            self.documents.push(RagDocument {
                id: id.clone(),
                content: chunk.clone(),
                source: source.to_string(),
                metadata: std::collections::HashMap::new(),
                chunk_index: i,
                created_at: now,
            });
            ids.push(id);
        }
        ids
    }

    pub fn ingest_with_metadata(
        &mut self,
        content: &str,
        source: &str,
        metadata: std::collections::HashMap<String, String>,
    ) -> Vec<String> {
        let chunks = chunk_text(content, &self.chunk_config);
        let now = chrono::Utc::now().timestamp();
        let mut ids = Vec::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let id = format!("{}_{}", source, i);
            self.documents.push(RagDocument {
                id: id.clone(),
                content: chunk.clone(),
                source: source.to_string(),
                metadata: metadata.clone(),
                chunk_index: i,
                created_at: now,
            });
            ids.push(id);
        }
        ids
    }

    pub fn query(&self, rag_query: &RagQuery) -> RagResult {
        let query_lower = rag_query.query.to_lowercase();
        let mut scored: Vec<ScoredDocument> = self
            .documents
            .iter()
            .filter(|doc| {
                rag_query
                    .source_filter
                    .as_ref()
                    .map_or(true, |f| &doc.source == f)
            })
            .filter_map(|doc| {
                let score = compute_relevance(&query_lower, &doc.content.to_lowercase());
                if score >= rag_query.min_score {
                    Some(ScoredDocument {
                        document: doc.clone(),
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(rag_query.top_k);
        let context = scored
            .iter()
            .map(|sd| {
                format!(
                    "[来源: {} (块 {})] {}",
                    sd.document.source, sd.document.chunk_index, sd.document.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        RagResult {
            query: rag_query.query.clone(),
            documents: scored,
            answer_context: context,
        }
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn sources(&self) -> Vec<&str> {
        let mut sources: Vec<&str> = self.documents.iter().map(|d| d.source.as_str()).collect();
        sources.sort();
        sources.dedup();
        sources
    }

    pub fn build_augmented_prompt(&self, query: &RagQuery, system_prefix: &str) -> String {
        let result = self.query(query);
        if result.answer_context.is_empty() {
            return format!(
                "{}\n\n用户问题：{}",
                system_prefix, query.query
            );
        }
        format!(
            "{}\n\n## 参考文档\n\n{}\n\n## 用户问题\n\n{}",
            system_prefix, result.answer_context, query.query
        )
    }
}

impl Default for RagPipeline {
    fn default() -> Self {
        Self::new()
    }
}

fn compute_relevance(query: &str, content: &str) -> f64 {
    let query_terms: Vec<&str> = query.split_whitespace().collect();
    if query_terms.is_empty() {
        return 0.0;
    }
    let mut matched = 0;
    for term in &query_terms {
        if content.contains(term) {
            matched += 1;
        }
    }
    let base_score = matched as f64 / query_terms.len() as f64;
    let length_bonus = if content.len() > 100 { 0.1 } else { 0.0 };
    (base_score + length_bonus).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_small() {
        let chunks = chunk_text("hello world", &ChunkConfig::default());
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_chunk_text_large() {
        let text: String = (0..1000).map(|i| format!("word{} ", i)).collect();
        let chunks = chunk_text(&text, &ChunkConfig { chunk_size: 100, overlap: 20 });
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= 120);
        }
    }

    #[test]
    fn test_rag_ingest() {
        let mut pipeline = RagPipeline::new();
        let ids = pipeline.ingest("这是一段测试文档的内容。", "test");
        assert!(!ids.is_empty());
        assert_eq!(pipeline.document_count(), ids.len());
    }

    #[test]
    fn test_rag_query() {
        let mut pipeline = RagPipeline::new();
        pipeline.ingest("体重 70kg，今天心情不错", "diary");
        pipeline.ingest("Python 是一种编程语言", "wiki");
        let result = pipeline.query(&RagQuery {
            query: "体重".to_string(),
            top_k: 3,
            min_score: 0.1,
            source_filter: None,
        });
        assert!(!result.documents.is_empty());
        assert!(result.documents[0].document.content.contains("体重"));
    }

    #[test]
    fn test_rag_source_filter() {
        let mut pipeline = RagPipeline::new();
        pipeline.ingest("体重 70kg", "diary");
        pipeline.ingest("Python 编程", "wiki");
        let result = pipeline.query(&RagQuery {
            query: "编程".to_string(),
            top_k: 3,
            min_score: 0.1,
            source_filter: Some("wiki".to_string()),
        });
        assert!(result.documents.iter().all(|d| d.document.source == "wiki"));
    }

    #[test]
    fn test_rag_sources() {
        let mut pipeline = RagPipeline::new();
        pipeline.ingest("content", "a");
        pipeline.ingest("content", "b");
        let sources = pipeline.sources();
        assert_eq!(sources.len(), 2);
    }

    #[test]
    fn test_compute_relevance() {
        let score = compute_relevance("体重", "体重 70kg");
        assert!(score > 0.0);
        let score_zero = compute_relevance("xyz", "abc");
        assert_eq!(score_zero, 0.0);
    }

    #[test]
    fn test_augmented_prompt() {
        let mut pipeline = RagPipeline::new();
        pipeline.ingest("用户体重 70kg", "health");
        let prompt = pipeline.build_augmented_prompt(
            &RagQuery {
                query: "体重".to_string(),
                top_k: 3,
                min_score: 0.1,
                source_filter: None,
            },
            "你是健康助手",
        );
        assert!(prompt.contains("参考文档"));
        assert!(prompt.contains("体重"));
    }
}
