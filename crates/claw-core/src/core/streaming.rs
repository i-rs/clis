use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProgress {
    pub tool_name: String,
    pub stage: ProgressStage,
    pub message: String,
    pub percentage: Option<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProgressStage {
    Started,
    Running,
    Completed,
    Failed,
}

#[allow(dead_code)]
pub struct ToolProgressEmitter {
    tool_name: String,
    tx: mpsc::UnboundedSender<ToolProgress>,
}

#[allow(dead_code)]
impl ToolProgressEmitter {
    pub fn new(tool_name: &str, tx: mpsc::UnboundedSender<ToolProgress>) -> Self {
        let emitter = Self {
            tool_name: tool_name.to_string(),
            tx,
        };
        emitter.emit(ProgressStage::Started, "开始执行", None);
        emitter
    }

    pub fn emit(&self, stage: ProgressStage, message: &str, percentage: Option<u8>) {
        let _ = self.tx.send(ToolProgress {
            tool_name: self.tool_name.clone(),
            stage,
            message: message.to_string(),
            percentage,
        });
    }

    pub fn progress(&self, message: &str, pct: u8) {
        self.emit(ProgressStage::Running, message, Some(pct.min(100)));
    }

    pub fn complete(&self, message: &str) {
        self.emit(ProgressStage::Completed, message, Some(100));
    }

    pub fn fail(&self, message: &str) {
        self.emit(ProgressStage::Failed, message, None);
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StreamingExecutionContext {
    pub progress_tx: Option<mpsc::UnboundedSender<ToolProgress>>,
}

#[allow(dead_code)]
impl StreamingExecutionContext {
    pub fn new() -> Self {
        Self { progress_tx: None }
    }

    pub fn with_progress(mut self, tx: mpsc::UnboundedSender<ToolProgress>) -> Self {
        self.progress_tx = Some(tx);
        self
    }

    pub fn emitter(&self, tool_name: &str) -> Option<ToolProgressEmitter> {
        self.progress_tx
            .clone()
            .map(|tx| ToolProgressEmitter::new(tool_name, tx))
    }
}

impl Default for StreamingExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn format_progress(progress: &ToolProgress) -> String {
    let stage_icon = match progress.stage {
        ProgressStage::Started => "⏳",
        ProgressStage::Running => "🔄",
        ProgressStage::Completed => "✅",
        ProgressStage::Failed => "❌",
    };
    match progress.percentage {
        Some(pct) => format!(
            "{} {} [{}%] {}",
            stage_icon, progress.tool_name, pct, progress.message
        ),
        None => format!("{} {} {}", stage_icon, progress.tool_name, progress.message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_format() {
        let p = ToolProgress {
            tool_name: "i_rs".to_string(),
            stage: ProgressStage::Running,
            message: "正在执行".to_string(),
            percentage: Some(50),
        };
        let formatted = format_progress(&p);
        assert!(formatted.contains("50%"));
        assert!(formatted.contains("i_rs"));
    }

    #[test]
    fn test_emitter() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let emitter = ToolProgressEmitter::new("test_tool", tx);
        emitter.progress("halfway", 50);
        emitter.complete("done");
        drop(emitter);
        let mut events = Vec::new();
        while let Some(p) = rx.blocking_recv() {
            events.push(p);
        }
        assert!(events.len() >= 3);
        assert_eq!(events[0].stage, ProgressStage::Started);
        assert_eq!(events[1].percentage, Some(50));
        assert_eq!(events[2].stage, ProgressStage::Completed);
    }

    #[test]
    fn test_streaming_context() {
        let ctx = StreamingExecutionContext::new();
        assert!(ctx.emitter("test").is_none());
        let (tx, _rx) = mpsc::unbounded_channel();
        let ctx = ctx.with_progress(tx);
        assert!(ctx.emitter("test").is_some());
    }
}
