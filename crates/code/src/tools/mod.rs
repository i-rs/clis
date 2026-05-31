pub mod bash;
pub mod batch_edit;
pub mod call_claw;
pub mod create_crate;
pub mod delete;
pub mod fs;
pub mod git;
pub mod lsp;
pub mod mcp;
pub mod pty;
pub mod register_tool;
pub mod rename;
pub mod skill;
pub mod test_runner;
pub mod verify;
pub mod web;

use crate::config::Config;
use async_trait::async_trait;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;

pub type ToolResult = anyhow::Result<String>;

pub use crate::error::ToolError;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    #[allow(dead_code)]
    fn description(&self) -> &str;
    fn schema(&self) -> Value;
    async fn call(&self, args: &Map<String, Value>) -> ToolResult;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new(_config: &Config) -> anyhow::Result<Self> {
        let mut tools: HashMap<String, Arc<dyn Tool>> = HashMap::new();

        let file_tools: Vec<Arc<dyn Tool>> = vec![
            Arc::new(fs::ReadTool),
            Arc::new(fs::WriteTool),
            Arc::new(fs::EditTool),
            Arc::new(fs::GlobTool),
            Arc::new(fs::GrepTool),
            Arc::new(fs::LsTool),
        ];
        for t in file_tools {
            tools.insert(t.name().to_string(), t);
        }

        tools.insert("bash".into(), Arc::new(bash::BashTool));
        tools.insert("git".into(), Arc::new(git::GitTool));
        tools.insert(
            "create_crate".into(),
            Arc::new(create_crate::CreateCrateTool),
        );
        tools.insert("call_claw".into(), Arc::new(call_claw::CallClawTool));
        tools.insert(
            "register_tool".into(),
            Arc::new(register_tool::RegisterTool),
        );
        tools.insert("web_fetch".into(), Arc::new(web::WebFetchTool));
        tools.insert("web_search".into(), Arc::new(web::WebSearchTool));
        tools.insert("delete".into(), Arc::new(delete::DeleteTool));
        tools.insert("rename".into(), Arc::new(rename::RenameTool));
        tools.insert("lsp_diagnostics".into(), Arc::new(lsp::LspDiagnosticsTool));
        tools.insert("lsp_definition".into(), Arc::new(lsp::LspDefinitionTool));
        tools.insert("lsp_references".into(), Arc::new(lsp::LspReferencesTool));
        tools.insert("lsp_hover".into(), Arc::new(lsp::LspHoverTool));
        tools.insert("lsp_rename".into(), Arc::new(lsp::LspRenameTool));
        tools.insert("lsp_symbols".into(), Arc::new(lsp::LspSymbolsTool));
        tools.insert("lsp_completion".into(), Arc::new(lsp::LspCompletionTool));
        tools.insert("pty_exec".into(), Arc::new(pty::PtyExecTool));
        tools.insert("pty_interrupt".into(), Arc::new(pty::PtyInterruptTool));
        tools.insert("mcp_connect".into(), Arc::new(mcp::McpConnectTool));
        tools.insert("verify".into(), Arc::new(verify::VerifyTool));
        tools.insert("batch_edit".into(), Arc::new(batch_edit::BatchEditTool));
        tools.insert("test".into(), Arc::new(test_runner::TestRunnerTool));
        tools.insert("skill".into(), Arc::new(skill::SkillTool));

        Ok(Self { tools })
    }

    #[allow(dead_code)]
    pub fn new_empty() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools
            .get(name)
            .cloned()
            .or_else(|| crate::tools::mcp::get_mcp_tool(name))
    }

    pub fn all_tools(&self) -> Vec<Arc<dyn Tool>> {
        let mut tools: Vec<_> = self.tools.values().cloned().collect();
        tools.extend(crate::tools::mcp::all_mcp_tools());
        tools
    }

    pub fn schemas(&self) -> Vec<Value> {
        self.all_tools().iter().map(|t| t.schema()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty_registers_nothing() {
        let reg = ToolRegistry::new_empty();
        assert!(reg.all_tools().is_empty());
        assert!(reg.schemas().is_empty());
        assert!(reg.get("read").is_none());
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = ToolRegistry::new_empty();
        let tool = Arc::new(super::fs::ReadTool);
        reg.register(tool);
        assert!(reg.get("read").is_some());
        assert_eq!(reg.all_tools().len(), 1);
        assert_eq!(reg.schemas().len(), 1);
    }

    #[test]
    fn test_register_overwrites() {
        let mut reg = ToolRegistry::new_empty();
        let tool1 = Arc::new(super::fs::ReadTool);
        let tool2 = Arc::new(super::bash::BashTool);
        reg.register(tool1);
        reg.register(tool2);
        assert_eq!(reg.all_tools().len(), 2);
        let bash_schema = reg
            .schemas()
            .into_iter()
            .find(|s| s["function"]["name"] == "bash");
        assert!(bash_schema.is_some());
    }
}
