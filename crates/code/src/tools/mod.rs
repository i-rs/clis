pub mod filesystem;
pub mod bash;
pub mod create_crate;
pub mod call_claw;
pub mod register_tool;
pub mod git;

use crate::config::Config;
use async_trait::async_trait;
use serde_json::{Value, Map};
use std::collections::HashMap;
use std::sync::Arc;

pub type ToolResult = anyhow::Result<String>;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
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
            Arc::new(filesystem::ReadTool),
            Arc::new(filesystem::WriteTool),
            Arc::new(filesystem::EditTool),
            Arc::new(filesystem::GlobTool),
            Arc::new(filesystem::GrepTool),
            Arc::new(filesystem::LsTool),
        ];
        for t in file_tools {
            tools.insert(t.name().to_string(), t);
        }

        tools.insert("bash".into(), Arc::new(bash::BashTool));
        tools.insert("git".into(), Arc::new(git::GitTool));
        tools.insert("create_crate".into(), Arc::new(create_crate::CreateCrateTool));
        tools.insert("call_claw".into(), Arc::new(call_claw::CallClawTool));
        tools.insert("register_tool".into(), Arc::new(register_tool::RegisterTool));

        Ok(Self { tools })
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn all_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.values().cloned().collect()
    }

    pub fn schemas(&self) -> Vec<Value> {
        self.tools.values().map(|t| t.schema()).collect()
    }
}
