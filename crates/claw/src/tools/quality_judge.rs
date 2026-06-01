use serde_json::Value;

const JUDGE_SYSTEM_PROMPT: &str = r#"你是一个回答质量评估器。根据用户问题、工具执行结果和最终回复，评估回复质量。

评估维度（每项 0-2 分）：
1. **相关性**: 回复是否切题回答了用户问题？
2. **准确性**: 回复中的信息是否与工具结果一致？
3. **完整性**: 回复是否充分回答了问题，没有遗漏关键信息？

输出 JSON 格式：
{"relevance": 0-2, "accuracy": 0-2, "completeness": 0-2, "reason": "简短说明"}

仅输出 JSON，不要其他文字。"#;

pub struct QualityJudgeRequest {
    pub user_query: String,
    pub tool_results: Vec<(String, String)>,
    pub final_response: String,
}

pub struct QualityJudgeResult {
    #[allow(dead_code)]
    pub relevance: u8,
    #[allow(dead_code)]
    pub accuracy: u8,
    #[allow(dead_code)]
    pub completeness: u8,
    #[allow(dead_code)]
    pub reason: String,
    #[allow(dead_code)]
    pub overall_score: f64,
}

impl QualityJudgeResult {
    pub fn from_json(json: &Value) -> Option<Self> {
        let relevance = json.get("relevance").and_then(|v| v.as_u64()).unwrap_or(2) as u8;
        let accuracy = json.get("accuracy").and_then(|v| v.as_u64()).unwrap_or(2) as u8;
        let completeness = json.get("completeness").and_then(|v| v.as_u64()).unwrap_or(2) as u8;
        let reason = json
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let overall_score = (relevance + accuracy + completeness) as f64 / 6.0;
        Some(Self {
            relevance,
            accuracy,
            completeness,
            reason,
            overall_score,
        })
    }
}

pub async fn judge_quality(
    http_client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    req: &QualityJudgeRequest,
) -> anyhow::Result<QualityJudgeResult> {
    let mut tool_summary = String::from("工具执行结果：\n");
    for (name, result) in &req.tool_results {
        let truncated = if result.len() > 500 {
            let s: String = result.chars().take(497).collect();
            format!("{}...", s)
        } else {
            result.clone()
        };
        tool_summary.push_str(&format!("- {}: {}\n", name, truncated));
    }

    let user_msg = format!(
        "用户问题：{}\n\n{}\n\n最终回复：{}",
        req.user_query, tool_summary, req.final_response
    );

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": JUDGE_SYSTEM_PROMPT},
            {"role": "user", "content": user_msg}
        ],
        "temperature": 0.0,
        "max_tokens": 200,
    });

    let response = http_client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Judge API error: {}", text));
    }

    let data: Value = response.json().await?;
    let content = data["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("{}");

    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let parsed: Value = serde_json::from_str(cleaned).unwrap_or_else(|_| {
        serde_json::json!({"relevance": 2, "accuracy": 2, "completeness": 2, "reason": "parse failed"})
    });

    QualityJudgeResult::from_json(&parsed)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse judge result"))
}
