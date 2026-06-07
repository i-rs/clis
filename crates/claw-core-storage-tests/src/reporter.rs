use std::path::Path;

/// Full test run report — aggregated across backends and conversations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestRunReport {
    pub timestamp: String,
    pub backends: Vec<BackendReport>,
}

/// Results for a single storage backend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackendReport {
    pub name: String,
    pub contracts: Vec<ContractResult>,
    pub duration_ms: u64,
}

/// Result of a single contract test against a backend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContractResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl TestRunReport {
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            backends: Vec::new(),
        }
    }

    pub fn add_backend(&mut self, report: BackendReport) {
        self.backends.push(report);
    }

    pub fn to_markdown(&self) -> String {
        let mut md = format!("# 存储层测试报告 — {}\n\n", self.timestamp);

        md.push_str("## 1. 契约测试结果\n\n");
        md.push_str("| 驱动 | 会话 | 消息日志 | 记忆 | 统计 | 工具缓存 | 技能 | 配置存储 |\n");
        md.push_str("|------|------|---------|------|------|---------|------|---------|\n");

        for backend in &self.backends {
            md.push_str(&format!("| {} ", backend.name));
            for contract in &backend.contracts {
                let icon = if contract.passed { "✅" } else { "❌" };
                md.push_str(&format!("| {} ", icon));
            }
            md.push_str("|\n");
        }

        md.push_str("\n## 2. 详细结果\n\n");
        for backend in &self.backends {
            md.push_str(&format!("### {}\n\n", backend.name));
            md.push_str(&format!("总耗时: {}ms\n\n", backend.duration_ms));
            md.push_str("| 契约 | 结果 | 耗时 | 错误 |\n");
            md.push_str("|------|------|------|------|\n");
            for c in &backend.contracts {
                let status = if c.passed { "✅ 通过" } else { "❌ 失败" };
                let err = c.error.as_deref().unwrap_or("-");
                md.push_str(&format!("| {} | {} | {}ms | {} |\n", c.name, status, c.duration_ms, err));
            }
            md.push_str("\n");
        }

        md
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn save(&self, reports_dir: &Path) -> std::io::Result<()> {
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let dir = reports_dir.join(ts.to_string());
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join("summary.md"), self.to_markdown())?;
        std::fs::write(dir.join("summary.json"), self.to_json())?;
        // Update latest symlink
        let latest = reports_dir.join("latest");
        let _ = std::fs::remove_file(&latest);
        #[cfg(unix)]
        std::os::unix::fs::symlink(ts.to_string(), &latest)?;
        Ok(())
    }
}

impl Default for TestRunReport {
    fn default() -> Self {
        Self::new()
    }
}
