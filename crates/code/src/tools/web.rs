use async_trait::async_trait;
use std::net::ToSocketAddrs;
use serde_json::{json, Value, Map};
use std::time::Instant;
use crate::config::Config;
use crate::runtime::LAST_WEB_REQUEST;
use crate::tools::{Tool, ToolResult};

const MAX_RESULTS: usize = 10;
const RATE_LIMIT_MS: u64 = 1000;

fn is_private_url(url: &str) -> bool {
    if url.starts_with("file://") || url.starts_with("ftp://") { return true; }
    let url_str = url.replace("http://", "").replace("https://", "");
    let host = url_str.split('/').next().unwrap_or("");

    let private_prefixes = [
        "10.", "172.16.", "172.17.", "172.18.", "172.19.", "172.20.", "172.21.",
        "172.22.", "172.23.", "172.24.", "172.25.", "172.26.", "172.27.", "172.28.",
        "172.29.", "172.30.", "172.31.", "192.168.", "127.", "169.254.",
        "localhost", "[::1]", "0.0.0.0",
    ];

    if private_prefixes.iter().any(|p| host.starts_with(p) || host == *p) {
        return true;
    }

    if let Ok(addrs) = (host, 0).to_socket_addrs() {
        for addr in addrs {
            let ip = addr.ip();
            let blocked = match ip {
                std::net::IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
                std::net::IpAddr::V6(v6) => v6.is_loopback(),
            };
            if blocked || ip.is_unspecified() {
                return true;
            }
        }
    }

    false
}

async fn rate_limit() {
    let mut last = LAST_WEB_REQUEST.lock().await;
    let elapsed = last.elapsed().as_millis() as u64;
    if elapsed < RATE_LIMIT_MS {
        tokio::time::sleep(std::time::Duration::from_millis(RATE_LIMIT_MS - elapsed)).await;
    }
    *last = Instant::now();
}

async fn search_duckduckgo(query: &str) -> anyhow::Result<String> {
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
        if results.len() >= MAX_RESULTS {
            break;
        }
    }
    if results.is_empty() {
        let snippet = &html[..html.len().min(2000)];
        return Ok(format!("No parsed results. Raw snippet:\n{}", snippet));
    }
    Ok(format!("DuckDuckGo results for '{}':\n  {}", query, results.join("\n  ")))
}

async fn search_serpapi(query: &str, api_key: &str) -> anyhow::Result<String> {
    let url = format!("https://serpapi.com/search?q={}&api_key={}&engine=google", urlencoding(query), api_key);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let resp = client.get(&url).send().await?;
    let data: Value = resp.json().await?;
    let empty = vec![];
    let items = data.get("organic_results").and_then(|v| v.as_array()).unwrap_or(&empty);
    let mut results = Vec::new();
    for item in items.iter().take(MAX_RESULTS) {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let snippet = item.get("snippet").and_then(|v| v.as_str()).unwrap_or("");
        results.push(format!("{} — {}", title, snippet));
    }
    if results.is_empty() {
        return Ok(format!("SerpAPI: no results for '{}'", query));
    }
    Ok(format!("SerpAPI results for '{}':\n  {}", query, results.join("\n  ")))
}

async fn search_bing(query: &str, api_key: &str) -> anyhow::Result<String> {
    let url = format!("https://api.bing.microsoft.com/v7.0/search?q={}", urlencoding(query));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let resp = client.get(&url)
        .header("Ocp-Apim-Subscription-Key", api_key)
        .send()
        .await?;
    let data: Value = resp.json().await?;
    let empty = vec![];
    let items = data.get("webPages").and_then(|v| v.get("value")).and_then(|v| v.as_array()).unwrap_or(&empty);
    let mut results = Vec::new();
    for item in items.iter().take(MAX_RESULTS) {
        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let snippet = item.get("snippet").and_then(|v| v.as_str()).unwrap_or("");
        results.push(format!("{} — {}", name, snippet));
    }
    if results.is_empty() {
        return Ok(format!("Bing: no results for '{}'", query));
    }
    Ok(format!("Bing results for '{}':\n  {}", query, results.join("\n  ")))
}

fn urlencoding(query: &str) -> String {
    query.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => "+".to_string(),
        c => format!("%{:02X}", c as u8),
    }).collect()
}

pub struct WebFetchTool;

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
        if is_private_url(url) {
            anyhow::bail!("Access to private/internal URLs is blocked");
        }
        rate_limit().await;
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

pub struct WebSearchTool;

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str { "web_search" }
    fn description(&self) -> &str { "Search the web. Supports SerpAPI, Bing, and DuckDuckGo (fallback). Configure via config.toml search_provider + search_api_key." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "Search the web for a query. Tries the configured provider first, falls back to DuckDuckGo.",
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
        rate_limit().await;

        let config = Config::load().ok();
        let provider = config.as_ref().and_then(|c| c.search_provider.as_deref()).unwrap_or("duckduckgo");
        let api_key = config.as_ref().and_then(|c| c.search_api_key.as_deref());

        match provider {
            "serpapi" | "serp" => {
                if let Some(key) = api_key {
                    return search_serpapi(query, key).await;
                }
            }
            "bing" | "bingsearch" => {
                if let Some(key) = api_key {
                    return search_bing(query, key).await;
                }
            }
            _ => {}
        }

        search_duckduckgo(query).await
    }
}
