use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};
use tokio::sync::Mutex;

static LSP_SESSION: Lazy<Mutex<crate::lsp::LspSession>> = Lazy::new(|| {
    Mutex::new(crate::lsp::LspSession::new())
});

pub struct LspDiagnosticsTool;
pub struct LspDefinitionTool;
pub struct LspReferencesTool;

#[async_trait]
impl Tool for LspDiagnosticsTool {
    fn name(&self) -> &str { "lsp_diagnostics" }
    fn description(&self) -> &str { "Get compiler diagnostics (errors/warnings) for a file using LSP." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "lsp_diagnostics",
                "description": "Get diagnostics for a file. Launches rust-analyzer as needed.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string", "description": "Path to the source file"}
                    },
                    "required": ["file_path"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let file_path = args.get("file_path").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("file_path required"))?;
        let mut session = LSP_SESSION.lock().await;
        let diags = session.get_diagnostics(file_path).await?;
        if diags.is_empty() {
            Ok(format!("{}: no diagnostics", file_path))
        } else {
            Ok(format!("Diagnostics for {}:\n{}", file_path, diags.join("\n")))
        }
    }
}

#[async_trait]
impl Tool for LspDefinitionTool {
    fn name(&self) -> &str { "lsp_definition" }
    fn description(&self) -> &str { "Go to definition of a symbol at a given position." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "lsp_definition",
                "description": "Find definition of a symbol at a file position (1-indexed line/column).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "line": {"type": "integer", "description": "Line number (1-indexed)"},
                        "character": {"type": "integer", "description": "Column number (1-indexed)"}
                    },
                    "required": ["file_path", "line", "character"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let file_path = args.get("file_path").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("file_path required"))?;
        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(1).saturating_sub(1) as u32;
        let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(1).saturating_sub(1) as u32;
        let mut session = LSP_SESSION.lock().await;
        let result = session.get_definition(file_path, line, character).await?;
        Ok(result)
    }
}

#[async_trait]
impl Tool for LspReferencesTool {
    fn name(&self) -> &str { "lsp_references" }
    fn description(&self) -> &str { "Find all references to a symbol at a given position." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "lsp_references",
                "description": "Find references to a symbol at a file position (1-indexed line/column).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "line": {"type": "integer", "description": "Line number (1-indexed)"},
                        "character": {"type": "integer", "description": "Column number (1-indexed)"}
                    },
                    "required": ["file_path", "line", "character"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let file_path = args.get("file_path").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("file_path required"))?;
        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(1).saturating_sub(1) as u32;
        let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(1).saturating_sub(1) as u32;
        let mut session = LSP_SESSION.lock().await;
        let result = session.get_references(file_path, line, character).await?;
        Ok(result)
    }
}
