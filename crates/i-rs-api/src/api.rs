use std::process::Command;
use axum::{
    Json,
    http::StatusCode,
};
use serde_json::json;

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
    let stdout = stdout.trim();

    if stdout.is_empty() {
        return Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "OK"
        })));
    }

    let json: serde_json::Value = serde_json::from_str(stdout)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON parse error: {}", e)))?;

    Ok(Json(json))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_cli_empty_output() {
        let result = run_cli("true", vec![]).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json.get("success").unwrap(), true);
    }
}
