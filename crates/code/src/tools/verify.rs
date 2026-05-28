use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};

pub struct VerifyTool;

#[async_trait]
impl Tool for VerifyTool {
    fn name(&self) -> &str { "verify" }
    fn description(&self) -> &str {
        "Run progressive verification: check -> clippy -> test -> fmt. Returns results at first failure."
    }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "verify",
                "description": "Run a chain of verification steps. Stops at the first failure. Steps: check, clippy, test, fmt.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "package": {"type": "string", "description": "Package name for -p flag, e.g. 'i-rs-code'. If omitted, uses --workspace."},
                        "skip_check": {"type": "boolean", "description": "Skip cargo check step"},
                        "skip_clippy": {"type": "boolean", "description": "Skip clippy step"},
                        "skip_test": {"type": "boolean", "description": "Skip test step"},
                        "skip_fmt": {"type": "boolean", "description": "Skip fmt check step"}
                    }
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let package = args.get("package").and_then(|v| v.as_str());
        let pkg_flag = match package {
            Some(p) => format!("-p {}", p),
            None => "--workspace".to_string(),
        };
        let skip_check = args.get("skip_check").and_then(|v| v.as_bool()).unwrap_or(false);
        let skip_clippy = args.get("skip_clippy").and_then(|v| v.as_bool()).unwrap_or(false);
        let skip_test = args.get("skip_test").and_then(|v| v.as_bool()).unwrap_or(false);
        let skip_fmt = args.get("skip_fmt").and_then(|v| v.as_bool()).unwrap_or(false);

        let mut report = Vec::new();
        let mut passed = 0;

        if !skip_check {
            report.push(format!("--- Step 1: cargo check {} ---", pkg_flag));
            let output = tokio::process::Command::new("cargo")
                .args(["check", &pkg_flag])
                .output()
                .await?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                passed += 1;
                report.push("PASS".to_string());
            } else {
                report.push(format!("FAIL\n{}", combine_output(&stdout, &stderr)));
                return Ok(report.join("\n"));
            }
        }

        if !skip_clippy {
            report.push(format!("--- Step 2: cargo clippy {} -- -D warnings ---", pkg_flag));
            let output = tokio::process::Command::new("cargo")
                .args(["clippy", &pkg_flag, "--", "-D", "warnings"])
                .output()
                .await?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                passed += 1;
                report.push("PASS".to_string());
            } else {
                report.push(format!("FAIL\n{}", combine_output(&stdout, &stderr)));
                return Ok(report.join("\n"));
            }
        }

        if !skip_test {
            report.push(format!("--- Step 3: cargo test {} ---", pkg_flag));
            let output = tokio::process::Command::new("cargo")
                .args(["test", &pkg_flag])
                .output()
                .await?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                passed += 1;
                report.push("PASS".to_string());
            } else {
                report.push(format!("FAIL\n{}", combine_output(&stdout, &stderr)));
                return Ok(report.join("\n"));
            }
        }

        if !skip_fmt {
            report.push("--- Step 4: cargo fmt --all --check ---".to_string());
            let output = tokio::process::Command::new("cargo")
                .args(["fmt", "--all", "--check"])
                .output()
                .await?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                passed += 1;
                report.push("PASS".to_string());
            } else {
                report.push(format!("FAIL\n{}", combine_output(&stdout, &stderr)));
                return Ok(report.join("\n"));
            }
        }

        report.push(format!("\nAll {} checks passed.", passed));

        Ok(report.join("\n"))
    }
}

fn combine_output(stdout: &str, stderr: &str) -> String {
    let mut out = String::new();
    if !stdout.is_empty() { out.push_str(stdout); }
    if !stderr.is_empty() { out.push('\n'); out.push_str(stderr); }
    out
}
