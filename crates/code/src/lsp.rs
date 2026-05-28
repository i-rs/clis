use lsp_types::*;
use std::sync::LazyLock;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};

static CLIENT_CAPS: LazyLock<ClientCapabilities> = LazyLock::new(ClientCapabilities::default);

pub struct LspSession {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    reader: Option<BufReader<tokio::process::ChildStdout>>,
    next_id: u32,
    initialized: bool,
    workspace_uri: lsp_types::Uri,
}

fn path_to_uri(file_path: &str) -> anyhow::Result<lsp_types::Uri> {
    let abs = std::fs::canonicalize(file_path)?;
    let s = format!("file://{}", abs.display());
    s.parse().map_err(|_| anyhow::anyhow!("invalid uri: {}", s))
}

impl LspSession {
    pub fn new() -> Self {
        let ws = std::env::current_dir()
            .map(|p| format!("file://{}", p.display()))
            .unwrap_or_else(|_| "file:///".into());
        let workspace_uri = ws.parse().unwrap_or_else(|_| "file:///".parse().unwrap());
        Self { process: None, stdin: None, reader: None, next_id: 1, initialized: false, workspace_uri }
    }

    async fn ensure_initialized(&mut self) -> anyhow::Result<()> {
        if self.initialized {
            return Ok(());
        }
        let mut child = Command::new("rust-analyzer")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("no stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout"))?;

        self.process = Some(child);
        self.stdin = Some(stdin);
        self.reader = Some(BufReader::new(stdout));

        let params = InitializeParams {
            process_id: Some(std::process::id()),
            capabilities: CLIENT_CAPS.clone(),
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: self.workspace_uri.clone(),
                name: "workspace".into(),
            }]),
            ..Default::default()
        };
        let _result: InitializeResult = self.send_request("initialize", params).await?;
        self.send_notification("initialized", serde_json::json!({})).await?;
        self.initialized = true;
        Ok(())
    }

    pub async fn get_diagnostics(&mut self, file_path: &str) -> anyhow::Result<Vec<String>> {
        self.ensure_initialized().await?;
        self.open_document(file_path).await?;
        let target_uri = path_to_uri(file_path)?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);

        loop {
            if std::time::Instant::now() > deadline {
                return Ok(Vec::new());
            }
            let msg = read_lsp_message(self.reader.as_mut().unwrap()).await?;
            if msg["method"] == "textDocument/publishDiagnostics"
                && let Some(uri_val) = msg["params"]["uri"].as_str() {
                    let msg_uri: lsp_types::Uri = uri_val.parse()?;
                    if msg_uri == target_uri {
                        let diags: Vec<Diagnostic> = serde_json::from_value(msg["params"]["diagnostics"].clone())?;
                        return Ok(diags.iter().map(|d| {
                            let sev = match d.severity {
                                Some(DiagnosticSeverity::ERROR) => "error",
                                Some(DiagnosticSeverity::WARNING) => "warning",
                                Some(DiagnosticSeverity::INFORMATION) => "info",
                                _ => "note",
                            };
                            let msg = &d.message;
                            format!("  {}:{}: {}: {}", d.range.start.line + 1, d.range.start.character + 1, sev, msg)
                        }).collect());
                    }
                }
        }
    }

    pub async fn get_definition(&mut self, file_path: &str, line: u32, character: u32) -> anyhow::Result<String> {
        self.ensure_initialized().await?;
        self.open_document(file_path).await?;
        let uri = path_to_uri(file_path)?;

        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line, character },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result: GotoDefinitionResponse = self.send_request("textDocument/definition", params).await?;
        Ok(format_locations(result))
    }

    pub async fn get_references(&mut self, file_path: &str, line: u32, character: u32) -> anyhow::Result<String> {
        self.ensure_initialized().await?;
        self.open_document(file_path).await?;
        let uri = path_to_uri(file_path)?;

        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line, character },
            },
            context: ReferenceContext { include_declaration: true },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result: Vec<Location> = self.send_request("textDocument/references", params).await?;
        if result.is_empty() {
            return Ok("No references found".into());
        }
        let lines: Vec<String> = result.iter().map(format_location).collect();
        Ok(format!("Found {} references:\n{}", lines.len(), lines.join("\n")))
    }

    async fn open_document(&mut self, file_path: &str) -> anyhow::Result<()> {
        let uri = path_to_uri(file_path)?;
        let content = tokio::fs::read_to_string(file_path).await?;
        let language_id = match file_path.rsplit('.').next().unwrap_or("") {
            "rs" => "rust",
            "ts" | "tsx" => "typescript",
            "js" | "jsx" => "javascript",
            "py" => "python",
            "go" => "go",
            "rb" => "ruby",
            _ => "plaintext",
        };
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: language_id.into(),
                version: 1,
                text: content,
            },
        };
        self.send_notification("textDocument/didOpen", params).await
    }

    async fn send_request<T: serde::Serialize, R: serde::de::DeserializeOwned>(
        &mut self, method: &str, params: T,
    ) -> anyhow::Result<R> {
        let id = self.next_id;
        self.next_id += 1;
        let request = serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        self.write_message(&request).await?;

        let reader = self.reader.as_mut().ok_or_else(|| anyhow::anyhow!("no reader"))?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);

        loop {
            if std::time::Instant::now() > deadline {
                anyhow::bail!("LSP request timed out (id={})", id);
            }
            let msg = read_lsp_message(reader).await?;
            if msg["id"].as_u64() == Some(id as u64) {
                if let Some(result) = msg.get("result") {
                    return Ok(serde_json::from_value(result.clone())?);
                }
                if let Some(error) = msg.get("error") {
                    anyhow::bail!("LSP error: {}", error["message"].as_str().unwrap_or("unknown"));
                }
                anyhow::bail!("LSP response missing result/error");
            }
        }
    }

    async fn send_notification<T: serde::Serialize>(&mut self, method: &str, params: T) -> anyhow::Result<()> {
        let notification = serde_json::json!({"jsonrpc": "2.0", "method": method, "params": params});
        self.write_message(&notification).await
    }

    async fn write_message(&mut self, msg: &Value) -> anyhow::Result<()> {
        let content = serde_json::to_string(msg)?;
        let header = format!("Content-Length: {}\r\n\r\n", content.len());
        if let Some(stdin) = &mut self.stdin {
            stdin.write_all(header.as_bytes()).await?;
            stdin.write_all(content.as_bytes()).await?;
            stdin.flush().await?;
        }
        Ok(())
    }
}

impl Drop for LspSession {
    fn drop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.start_kill();
        }
    }
}

async fn read_lsp_message(reader: &mut BufReader<tokio::process::ChildStdout>) -> anyhow::Result<Value> {
    let mut content_len: Option<usize> = None;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line).await?;
        if bytes == 0 {
            anyhow::bail!("LSP server closed connection");
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
            content_len = Some(len_str.parse()?);
        }
    }
    let len = content_len.ok_or_else(|| anyhow::anyhow!("missing Content-Length"))?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    let content = String::from_utf8(buf)?;
    Ok(serde_json::from_str(&content)?)
}

fn format_location(loc: &Location) -> String {
    #[allow(clippy::to_string_in_format_args)]
    let uri_str = loc.uri.to_string();
    format!("{}:{}:{}-{}:{}",
        uri_str,
        loc.range.start.line + 1, loc.range.start.character + 1,
        loc.range.end.line + 1, loc.range.end.character + 1)
}

fn format_locations(response: GotoDefinitionResponse) -> String {
    match response {
        GotoDefinitionResponse::Scalar(loc) => format_location(&loc),
        GotoDefinitionResponse::Array(locs) => {
            locs.iter().map(format_location).collect::<Vec<_>>().join("\n")
        }
        _ => "No definition found".into(),
    }
}
