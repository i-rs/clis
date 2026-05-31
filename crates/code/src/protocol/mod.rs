pub mod handler;
pub mod transport;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClawTask {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub task_id: String,
    pub prompt: Option<String>,
    pub context: Option<Value>,
    pub request_id: Option<String>,
    pub content: Option<String>,
    pub modification: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CodeEvent {
    pub event: String,
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
}

impl CodeEvent {
    pub fn progress(task_id: &str, stage: &str, detail: &str) -> Self {
        Self {
            event: "progress".into(),
            task_id: task_id.into(),
            content: Some(detail.into()),
            request_id: None,
            r#type: None,
            detail: None,
            tool: None,
            stage: Some(stage.into()),
        }
    }

    pub fn request(
        task_id: &str,
        request_id: &str,
        r#type: &str,
        content: &str,
        detail: Option<Value>,
    ) -> Self {
        Self {
            event: "request".into(),
            task_id: task_id.into(),
            content: Some(content.into()),
            request_id: Some(request_id.into()),
            r#type: Some(r#type.into()),
            detail,
            tool: None,
            stage: None,
        }
    }

    pub fn tool_created(task_id: &str, tool: Value) -> Self {
        Self {
            event: "tool_created".into(),
            task_id: task_id.into(),
            content: None,
            request_id: None,
            r#type: None,
            detail: None,
            tool: Some(tool),
            stage: None,
        }
    }

    pub fn done(task_id: &str, content: &str) -> Self {
        Self {
            event: "done".into(),
            task_id: task_id.into(),
            content: Some(content.into()),
            request_id: None,
            r#type: None,
            detail: None,
            tool: None,
            stage: None,
        }
    }

    #[allow(dead_code)]
    pub fn error(task_id: &str, content: &str) -> Self {
        Self {
            event: "error".into(),
            task_id: task_id.into(),
            content: Some(content.into()),
            request_id: None,
            r#type: None,
            detail: None,
            tool: None,
            stage: None,
        }
    }
}
