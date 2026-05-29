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
                        "skip_fmt": {"type": "boolean", "description": "Skip fmt check step"},
                        "timeout_secs": {"type": "integer", "description": "Timeout per step in seconds (default: 300)"}
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
        let timeout_secs = args.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(300);
        let timeout = std::time::Duration::from_secs(timeout_secs);

        let mut report = Vec::new();
        let mut passed = 0;

        if !skip_check {
            report.push(format!("--- Step 1: cargo check {} ---", pkg_flag));
            match run_cargo(&["check", &pkg_flag], timeout).await {
                StepResult::Pass => { passed += 1; report.push("PASS".into()); }
                StepResult::Fail(out) => { report.push(format!("FAIL\n{}", out)); return Ok(report.join("\n")); }
                StepResult::Timeout => { report.push(format!("TIMEOUT after {}s", timeout_secs)); return Ok(report.join("\n")); }
            }
        }

        if !skip_clippy {
            report.push(format!("--- Step 2: cargo clippy {} -- -D warnings ---", pkg_flag));
            match run_cargo(&["clippy", &pkg_flag, "--", "-D", "warnings"], timeout).await {
                StepResult::Pass => { passed += 1; report.push("PASS".into()); }
                StepResult::Fail(out) => { report.push(format!("FAIL\n{}", out)); return Ok(report.join("\n")); }
                StepResult::Timeout => { report.push(format!("TIMEOUT after {}s", timeout_secs)); return Ok(report.join("\n")); }
            }
        }

        if !skip_test {
            report.push(format!("--- Step 3: cargo test {} ---", pkg_flag));
            match run_cargo(&["test", &pkg_flag], timeout).await {
                StepResult::Pass => { passed += 1; report.push("PASS".into()); }
                StepResult::Fail(out) => { report.push(format!("FAIL\n{}", out)); return Ok(report.join("\n")); }
                StepResult::Timeout => { report.push(format!("TIMEOUT after {}s", timeout_secs)); return Ok(report.join("\n")); }
            }
        }

        if !skip_fmt {
            report.push("--- Step 4: cargo fmt --all --check ---".into());
            match run_cargo(&["fmt", "--all", "--check"], timeout).await {
                StepResult::Pass => { passed += 1; report.push("PASS".into()); }
                StepResult::Fail(out) => { report.push(format!("FAIL\n{}", out)); return Ok(report.join("\n")); }
                StepResult::Timeout => { report.push(format!("TIMEOUT after {}s", timeout_secs)); return Ok(report.join("\n")); }
            }
        }

        report.push(format!("\nAll {} checks passed.", passed));
        Ok(report.join("\n"))
    }
}

enum StepResult {
    Pass,
    Fail(String),
    Timeout,
}

async fn run_cargo(args: &[&str], timeout: std::time::Duration) -> StepResult {
    let result = tokio::time::timeout(timeout,
        tokio::process::Command::new("cargo").args(args).output()
    ).await;

    match result {
        Err(_) => StepResult::Timeout,
        Ok(Err(e)) => StepResult::Fail(format!("IO error: {}", e)),
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                StepResult::Pass
            } else {
                StepResult::Fail(combine_output(&stdout, &stderr))
            }
        }
    }
}

fn combine_output(stdout: &str, stderr: &str) -> String {
    let mut out = String::new();
    if !stdout.is_empty() { out.push_str(stdout); }
    if !stderr.is_empty() { out.push('\n'); out.push_str(stderr); }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_output_both_nonempty() {
        let result = combine_output("hello", "world");
        assert_eq!(result, "hello\nworld");
    }

    #[test]
    fn test_combine_output_stdout_only() {
        let result = combine_output("hello", "");
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_combine_output_stderr_only() {
        let result = combine_output("", "error: something failed");
        assert_eq!(result, "\nerror: something failed");
    }

    #[test]
    fn test_combine_output_both_empty() {
        let result = combine_output("", "");
        assert_eq!(result, "");
    }
}
