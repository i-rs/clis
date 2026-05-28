use async_trait::async_trait;
use serde_json::{json, Map, Value};
use crate::tools::{Tool, ToolResult};

pub struct TestRunnerTool;

#[async_trait]
impl Tool for TestRunnerTool {
    fn name(&self) -> &str { "test" }
    fn description(&self) -> &str {
        "Run tests for the current project. Supports filtering by package, test name, and output format."
    }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "test",
                "description": "Run cargo test with optional filters.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "package": {"type": "string", "description": "Package to test (e.g. i-rs-code). If omitted, tests current package."},
                        "test_name": {"type": "string", "description": "Optional test name filter (partial match)."},
                        "features": {"type": "string", "description": "Feature flags (comma separated)"},
                        "no_capture": {"type": "boolean", "description": "Disable test capture (default: true)"}
                    }
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let package = args.get("package").and_then(|v| v.as_str());
        let test_name = args.get("test_name").and_then(|v| v.as_str());
        let features = args.get("features").and_then(|v| v.as_str());
        let no_capture = args.get("no_capture").and_then(|v| v.as_bool()).unwrap_or(true);

        let mut cmd = tokio::process::Command::new("cargo");
        cmd.arg("test");

        if let Some(pkg) = package {
            cmd.args(["-p", pkg]);
        }

        if let Some(name) = test_name {
            cmd.arg("--").arg(name);
        }

        if let Some(feat) = features {
            cmd.args(["--features", feat]);
        }

        if no_capture {
            cmd.arg("--nocapture");
        }

        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = String::new();
        result.push_str("$ cargo test");
        if let Some(pkg) = package { result.push_str(&format!(" -p {}", pkg)); }
        if let Some(name) = test_name { result.push_str(&format!(" -- {}", name)); }
        result.push('\n');

        if !stdout.is_empty() {
            for line in stdout.lines() {
                if line.starts_with("test ") || line.starts_with("running ") || line.contains("FAILED") || line.starts_with("error") {
                    result.push_str(line);
                    result.push('\n');
                }
            }
            if let Some(summary) = stdout.lines().rev().find(|l| l.starts_with("test result")) {
                result.push_str(summary);
                result.push('\n');
            }
        }
        if !stderr.is_empty() && !output.status.success() {
            result.push_str(&stderr);
        }
        if !output.status.success() {
            result.push_str(&format!("exit code: {}", output.status.code().unwrap_or(-1)));
        }

        Ok(result)
    }
}
