use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

/// Built-in tool that performs web searches.
///
/// Uses DuckDuckGo Instant Answer API by default (free, no API key required).
/// If the user has configured a `search_base_url` and `search_api_key`, uses
/// those for a custom search backend instead.
pub struct WebSearchTool;

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

    fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let query = args
            .get("query")
            .and_then(|q| q.as_str())
            .unwrap_or("")
            .trim();
        if query.is_empty() {
            return Err("Please provide a search query".to_string());
        }

        // Load config to check for custom search settings
        let cfg = crate::config::Config::load().map_err(|e| format!("加载配置失败: {}", e))?;

        if let Some(custom_url) = &cfg.search_base_url {
            search_custom(&custom_url, &cfg.search_api_key, query)
        } else {
            search_duckduckgo(query)
        }
    }
}

/// Search using DuckDuckGo Instant Answer API (free, no API key).
fn search_duckduckgo(query: &str) -> Result<String, String> {
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencode(query)
    );

    let resp = reqwest::blocking::get(&url)
        .map_err(|e| format!("搜索请求失败: {}", e))?;

    let data: Value = resp
        .json()
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let mut output = String::new();

    // Abstract
    if let Some(abstract_text) = data.get("AbstractText").and_then(|v| v.as_str()) {
        if !abstract_text.is_empty() {
            output.push_str(&format!("📝 摘要: {}\n", abstract_text));
            if let Some(src) = data.get("AbstractSource").and_then(|v| v.as_str()) {
                if !src.is_empty() {
                    if let Some(url) = data.get("AbstractURL").and_then(|v| v.as_str()) {
                        output.push_str(&format!("   来源: {} ({})\n", src, url));
                    }
                }
            }
            output.push('\n');
        }
    }

    // Direct answer
    if let Some(answer) = data.get("Answer").and_then(|v| v.as_str()) {
        if !answer.is_empty() {
            output.push_str(&format!("✅ 答案: {}\n\n", answer));
        }
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
                let url = topic
                    .get("FirstURL")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
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
                        let url = sub
                            .get("FirstURL")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
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
fn search_custom(base_url: &str, api_key: &Option<String>, query: &str) -> Result<String, String> {
    let separator = if base_url.contains('?') { "&" } else { "?" };
    let url = format!("{}{}q={}", base_url, separator, urlencode(query));

    let client = reqwest::blocking::Client::new();
    let mut req = client.get(&url);

    if let Some(key) = api_key {
        if !key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
    }

    let resp = req
        .send()
        .map_err(|e| format!("自定义搜索请求失败: {}", e))?;

    let text = resp
        .text()
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
    let result_keys = ["results", "items", "organic", "organic_results", "web", "entries"];
    let title_keys = ["title", "Title", "name", "Name", "heading", "Heading"];
    let snippet_keys = ["snippet", "Snippet", "description", "Description", "text", "Text", "abstract", "Abstract"];
    let url_keys = ["url", "Url", "URL", "link", "Link", "href", "Href", "firstURL", "FirstURL"];

    // Check if this looks like a result item
    for tk in &title_keys {
        if let Some(title) = data.get(*tk).and_then(|v| v.as_str()) {
            if title.is_empty() {
                continue;
            }
            output.push_str(&format!("{}• {}\n", prefix, title));

            for sk in &snippet_keys {
                if let Some(snippet) = data.get(*sk).and_then(|v| v.as_str()) {
                    if !snippet.is_empty() {
                        output.push_str(&format!("{}  {}\n", prefix, snippet));
                        break;
                    }
                }
            }

            for uk in &url_keys {
                if let Some(url) = data.get(*uk).and_then(|v| v.as_str()) {
                    if !url.is_empty() {
                        output.push_str(&format!("{}  {}\n", prefix, url));
                        break;
                    }
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
