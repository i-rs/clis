use std::process::Command;
use axum::{
    Json,
    http::StatusCode,
};

pub async fn run_cli(
    binary: &str,
    args: Vec<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let output = Command::new(binary)
        .arg("--json")
        .args(&args)
        .output()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to execute {binary}: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Command failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON parse error: {}", e)))?;

    Ok(Json(json))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_cli() {
        let result = run_cli("echo", vec!["test".to_string()]).await;
        assert!(result.is_ok());
    }
}
