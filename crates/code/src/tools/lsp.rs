use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};
use crate::runtime::LSP_SESSION;

pub struct LspDiagnosticsTool;
pub struct LspDefinitionTool;
pub struct LspReferencesTool;
pub struct LspHoverTool;
pub struct LspRenameTool;
pub struct LspSymbolsTool;

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

#[async_trait]
impl Tool for LspHoverTool {
    fn name(&self) -> &str { "lsp_hover" }
    fn description(&self) -> &str { "Get hover documentation for a symbol at a given position." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "lsp_hover",
                "description": "Get type signature and documentation for a symbol at a file position.",
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
        let result = session.get_hover(file_path, line, character).await?;
        Ok(result)
    }
}

#[async_trait]
impl Tool for LspRenameTool {
    fn name(&self) -> &str { "lsp_rename" }
    fn description(&self) -> &str { "Rename a symbol across the entire workspace using LSP." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "lsp_rename",
                "description": "Rename a symbol at a given position across all files in the workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "line": {"type": "integer", "description": "Line number (1-indexed)"},
                        "character": {"type": "integer", "description": "Column number (1-indexed)"},
                        "new_name": {"type": "string", "description": "The new name for the symbol"}
                    },
                    "required": ["file_path", "line", "character", "new_name"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let file_path = args.get("file_path").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("file_path required"))?;
        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(1).saturating_sub(1) as u32;
        let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(1).saturating_sub(1) as u32;
        let new_name = args.get("new_name").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("new_name required"))?;
        let mut session = LSP_SESSION.lock().await;
        let result = session.get_rename(file_path, line, character, new_name).await?;
        Ok(result)
    }
}

#[async_trait]
impl Tool for LspSymbolsTool {
    fn name(&self) -> &str { "lsp_symbols" }
    fn description(&self) -> &str { "Get document symbols (outline) for a file." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "lsp_symbols",
                "description": "Get the symbol outline (functions, structs, traits, etc.) for a source file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"}
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
        let result = session.get_document_symbols(file_path).await?;
        Ok(result)
    }
}
