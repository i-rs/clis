use std::process::Command;

pub fn execute(tool: &str, cmd: &str, args: &[String]) -> Result<String, String> {
    let output = Command::new("i-rs")
        .arg(tool)
        .arg(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("执行 i-rs {} {} 失败: {}", tool, cmd, e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            Ok("ok".to_string())
        } else {
            Ok(trimmed.to_string())
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = if stderr.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        Err(combined)
    }
}
