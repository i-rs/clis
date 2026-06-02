use crate::core::rag::{RagPipeline, RagQuery};
use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;
use std::sync::Mutex;

pub struct RagTool {
    pipeline: Mutex<RagPipeline>,
}

impl RagTool {
    pub fn new() -> Self {
        Self {
            pipeline: Mutex::new(RagPipeline::new()),
        }
    }
}

#[async_trait::async_trait]
impl ClawTool for RagTool {
    fn name(&self) -> &str {
        "rag"
    }

    fn description(&self) -> &str {
        "检索增强生成工具。可摄入文档内容并进行语义检索，用于回答基于文档的问题。\n\
         支持操作：ingest (摄入文档), query (检索), sources (查看来源), count (文档数), augmented (生成增强提示)。"
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["ingest", "query", "sources", "count", "augmented"],
                    "description": "操作类型"
                },
                "content": {
                    "type": "string",
                    "description": "文档内容 (ingest 时必填)"
                },
                "source": {
                    "type": "string",
                    "description": "文档来源标识 (ingest 时必填)"
                },
                "query": {
                    "type": "string",
                    "description": "搜索查询 (query/augmented 时必填)"
                },
                "top_k": {
                    "type": "integer",
                    "description": "返回结果数量 (默认 5)",
                    "default": 5
                },
                "min_score": {
                    "type": "number",
                    "description": "最低相关度分数 (默认 0.1)",
                    "default": 0.1
                },
                "system_prefix": {
                    "type": "string",
                    "description": "系统提示前缀 (augmented 时使用)"
                }
            },
            "required": ["action"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClawError::Validation("缺少 action 参数".to_string()))?;

        let mut pipeline = self.pipeline.lock().map_err(|e| ClawError::Execution(e.to_string()))?;

        match action {
            "ingest" => {
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ClawError::Validation("ingest 需要 content 参数".to_string()))?;
                let source = args
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let ids = pipeline.ingest(content, source);
                Ok(format!(
                    "已摄入 {} 个文档块 (来源: {})",
                    ids.len(),
                    source
                ))
            }
            "query" => {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ClawError::Validation("query 需要 query 参数".to_string()))?;
                let top_k = args
                    .get("top_k")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as usize;
                let min_score = args
                    .get("min_score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.1);
                let result = pipeline.query(&RagQuery {
                    query: query.to_string(),
                    top_k,
                    min_score,
                    source_filter: None,
                });
                if result.documents.is_empty() {
                    Ok("未找到相关文档。请先使用 ingest 摄入文档。".to_string())
                } else {
                    Ok(result.answer_context)
                }
            }
            "sources" => {
                let sources = pipeline.sources();
                if sources.is_empty() {
                    Ok("暂无文档来源。请先使用 ingest 摄入文档。".to_string())
                } else {
                    Ok(format!("已摄入文档来源: {}", sources.join(", ")))
                }
            }
            "count" => {
                Ok(format!("当前文档块数量: {}", pipeline.document_count()))
            }
            "augmented" => {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ClawError::Validation("augmented 需要 query 参数".to_string()))?;
                let top_k = args
                    .get("top_k")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as usize;
                let min_score = args
                    .get("min_score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.1);
                let prefix = args
                    .get("system_prefix")
                    .and_then(|v| v.as_str())
                    .unwrap_or("请根据以下参考文档回答用户问题。");
                let prompt = pipeline.build_augmented_prompt(
                    &RagQuery {
                        query: query.to_string(),
                        top_k,
                        min_score,
                        source_filter: None,
                    },
                    prefix,
                );
                Ok(prompt)
            }
            _ => Err(ClawError::Validation(format!(
                "未知操作: {}。支持: ingest, query, sources, count, augmented",
                action
            ))),
        }
    }
}
