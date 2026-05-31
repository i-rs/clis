use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

/// Built-in tool that performs web searches.
///
/// Uses DuckDuckGo Instant Answer API by default (free, no API key required).
/// If the user has configured a `search_base_url` and `search_api_key`, uses
/// those for a custom search backend instead.
pub struct WebSearchTool;

#[async_trait::async_trait]
impl ClawTool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for current information. Use this when the user asks \
         about recent events, news, facts you don't know, or anything that \
         requires up-to-date information from the internet."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query — be specific for best results"
                }
            },
            "required": ["query"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError> {
        let query = args
            .get("query")
            .and_then(|q| q.as_str())
            .unwrap_or("")
            .trim();
        if query.is_empty() {
            return Err(ClawError::Validation(
                "Please provide a search query".to_string(),
            ));
        }

        if let Some(custom_url) = &ctx.config.search_base_url {
            search_custom(
                custom_url,
                &ctx.config.search_api_key,
                query,
                &ctx.http_client,
            )
            .await
        } else {
            search_duckduckgo(query, &ctx.http_client).await
        }
    }
}

/// Search using DuckDuckGo Instant Answer API (free, no API key).
async fn search_duckduckgo(query: &str, client: &reqwest::Client) -> Result<String, ClawError> {
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencode(query)
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("搜索请求失败: {}", e))?;

    let data: Value = resp
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let mut output = String::new();

    // Abstract
    if let Some(abstract_text) = data.get("AbstractText").and_then(|v| v.as_str())
        && !abstract_text.is_empty()
    {
        output.push_str(&format!("📝 摘要: {}\n", abstract_text));
        if let Some(src) = data.get("AbstractSource").and_then(|v| v.as_str())
            && !src.is_empty()
            && let Some(url) = data.get("AbstractURL").and_then(|v| v.as_str())
        {
            output.push_str(&format!("   来源: {} ({})\n", src, url));
        }
        output.push('\n');
    }

    // Direct answer
    if let Some(answer) = data.get("Answer").and_then(|v| v.as_str())
        && !answer.is_empty()
    {
        output.push_str(&format!("✅ 答案: {}\n\n", answer));
    }

    // Related topics (these contain the actual search results)
    if let Some(topics) = data.get("RelatedTopics").and_then(|v| v.as_array()) {
        let mut count = 0;
        for topic in topics {
            if count >= 8 {
                break;
            }

            // Direct topic
            if let Some(text) = topic.get("Text").and_then(|v| v.as_str()) {
                let url = topic.get("FirstURL").and_then(|v| v.as_str()).unwrap_or("");
                output.push_str(&format!("• {}\n", text));
                if !url.is_empty() {
                    output.push_str(&format!("  {}\n", url));
                }
                output.push('\n');
                count += 1;
                continue;
            }

            // Category with sub-topics
            if let Some(sub_topics) = topic.get("Topics").and_then(|v| v.as_array()) {
                for sub in sub_topics {
                    if count >= 8 {
                        break;
                    }
                    if let Some(text) = sub.get("Text").and_then(|v| v.as_str()) {
                        let url = sub.get("FirstURL").and_then(|v| v.as_str()).unwrap_or("");
                        output.push_str(&format!("• {}\n", text));
                        if !url.is_empty() {
                            output.push_str(&format!("  {}\n", url));
                        }
                        output.push('\n');
                        count += 1;
                    }
                }
            }
        }
    }

    if output.is_empty() {
        // Try extracting from Results array as fallback
        if let Some(results) = data.get("Results").and_then(|v| v.as_array()) {
            for (i, r) in results.iter().enumerate() {
                if i >= 5 {
                    break;
                }
                let text = r.get("Text").and_then(|v| v.as_str()).unwrap_or("");
                let url = r.get("FirstURL").and_then(|v| v.as_str()).unwrap_or("");
                if !text.is_empty() {
                    output.push_str(&format!("• {}\n", text));
                    if !url.is_empty() {
                        output.push_str(&format!("  {}\n", url));
                    }
                    output.push('\n');
                }
            }
        }
    }

    if output.is_empty() {
        Ok(format!("未找到关于 \"{}\" 的搜索结果", query))
    } else {
        Ok(output.trim().to_string())
    }
}

/// Search using a custom search API endpoint.
/// The URL should accept query parameter `?q=QUERY`.
/// If api_key is set, adds `Authorization: Bearer <key>` header.
async fn search_custom(
    base_url: &str,
    api_key: &Option<String>,
    query: &str,
    client: &reqwest::Client,
) -> Result<String, ClawError> {
    let separator = if base_url.contains('?') { "&" } else { "?" };
    let url = format!("{}{}q={}", base_url, separator, urlencode(query));

    let mut req = client.get(&url);

    if let Some(key) = api_key
        && !key.is_empty()
    {
        req = req.header("Authorization", format!("Bearer {}", key));
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("自定义搜索请求失败: {}", e))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    // Try parsing as JSON for structured output
    if let Ok(data) = serde_json::from_str::<Value>(&text) {
        // Try common search API response formats
        let mut output = String::new();
        try_extract_results(&data, &mut output, "", 0);

        if !output.is_empty() {
            return Ok(output.trim().to_string());
        }
    }

    // Fallback: return raw response (truncated)
    let preview: String = text.chars().take(2000).collect();
    Ok(preview)
}

/// Recursively try to extract search results from a JSON response.
/// Handles various API formats (Google, SerpAPI, custom, etc.).
fn try_extract_results(data: &Value, output: &mut String, prefix: &str, depth: usize) {
    if depth > 3 {
        return;
    }

    // Common result field names across search APIs
    let result_keys = [
        "results",
        "items",
        "organic",
        "organic_results",
        "web",
        "entries",
    ];
    let title_keys = ["title", "Title", "name", "Name", "heading", "Heading"];
    let snippet_keys = [
        "snippet",
        "Snippet",
        "description",
        "Description",
        "text",
        "Text",
        "abstract",
        "Abstract",
    ];
    let url_keys = [
        "url", "Url", "URL", "link", "Link", "href", "Href", "firstURL", "FirstURL",
    ];

    // Check if this looks like a result item
    for tk in &title_keys {
        if let Some(title) = data.get(*tk).and_then(|v| v.as_str()) {
            if title.is_empty() {
                continue;
            }
            output.push_str(&format!("{}• {}\n", prefix, title));

            for sk in &snippet_keys {
                if let Some(snippet) = data.get(*sk).and_then(|v| v.as_str())
                    && !snippet.is_empty()
                {
                    output.push_str(&format!("{}  {}\n", prefix, snippet));
                    break;
                }
            }

            for uk in &url_keys {
                if let Some(url) = data.get(*uk).and_then(|v| v.as_str())
                    && !url.is_empty()
                {
                    output.push_str(&format!("{}  {}\n", prefix, url));
                    break;
                }
            }

            output.push('\n');
            return;
        }
    }

    // Search for arrays that might contain results
    if let Some(obj) = data.as_object() {
        for (_key, val) in obj {
            if let Some(arr) = val.as_array() {
                for item in arr {
                    try_extract_results(item, output, prefix, depth + 1);
                }
            }
        }
    }

    // Search for named result arrays
    for rk in &result_keys {
        if let Some(arr) = data.get(*rk).and_then(|v| v.as_array()) {
            for item in arr {
                try_extract_results(item, output, prefix, depth + 1);
            }
        }
    }
}

fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            other => {
                let bytes = other.to_string().into_bytes();
                bytes
                    .iter()
                    .map(|&b| format!("%{:02X}", b))
                    .collect::<String>()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencode_ascii() {
        assert_eq!(urlencode("hello"), "hello");
    }

    #[test]
    fn test_urlencode_spaces() {
        assert_eq!(urlencode("hello world"), "hello+world");
    }

    #[test]
    fn test_urlencode_special_chars() {
        assert_eq!(urlencode("a&b=c"), "a%26b%3Dc");
    }

    #[test]
    fn test_urlencode_unicode() {
        assert_eq!(urlencode("中文"), "%E4%B8%AD%E6%96%87");
    }

    #[test]
    fn test_urlencode_mixed() {
        assert_eq!(
            urlencode("hello 世界 & test"),
            "hello+%E4%B8%96%E7%95%8C+%26+test"
        );
    }

    #[test]
    fn test_urlencode_empty() {
        assert_eq!(urlencode(""), "");
    }

    #[test]
    fn test_urlencode_safe_chars() {
        // RFC 3986 unreserved characters: A-Z, a-z, 0-9, -, ., _, ~
        assert_eq!(urlencode("abc123-._~"), "abc123-._~");
    }
}
