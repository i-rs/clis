use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};

pub struct WebFetchTool;
pub struct WebSearchTool;

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str { "web_fetch" }
    fn description(&self) -> &str { "Fetch a URL and return its text content" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "web_fetch",
                "description": "Fetch a URL and return its text content",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": {"type": "string", "description": "URL to fetch"}
                    },
                    "required": ["url"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let url = args.get("url").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("url required"))?;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        let resp = client.get(url).send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        let truncated = if text.len() > 50000 {
            format!("{}...\n[truncated {} bytes]", &text[..50000], text.len() - 50000)
        } else {
            text
        };
        Ok(format!("Status: {}\n\n{}", status, truncated))
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str { "web_search" }
    fn description(&self) -> &str { "Search the web using a simple HTML scraping approach (DuckDuckGo)" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "Search the web for a query",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query"}
                    },
                    "required": ["query"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("query required"))?;
        let encoded: String = query.chars().map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            c => format!("%{:02X}", c as u8),
        }).collect();
        let url = format!("https://html.duckduckgo.com/html/?q={}", encoded);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        let resp = client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .send()
            .await?;
        let html = resp.text().await?;
        let mut results = Vec::new();
        let link_re = regex::Regex::new(r###"<a[^>]*class="result__a"[^>]*>(.*?)</a>"###).unwrap();
        let tag_re = regex::Regex::new("<[^>]*>").unwrap();
        for cap in link_re.captures_iter(&html) {
            let title = cap[1].to_string();
            let clean = tag_re.replace_all(&title, "");
            results.push(clean.to_string());
            if results.len() >= 10 {
                break;
            }
        }
        if results.is_empty() {
            let snippet = &html[..html.len().min(2000)];
            return Ok(format!("No parsed results for '{}'. Raw snippet:\n{}", query, snippet));
        }
        Ok(format!("Search results for '{}':\n  {}", query, results.join("\n  ")))
    }
}
