use lsp_types::*;
use std::sync::LazyLock;
use serde_json::Value;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};

const LSP_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const LSP_DIAGNOSTICS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

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

fn detect_language(file_path: &str) -> &'static str {
    match file_path.rsplit('.').next().unwrap_or("") {
        "rs" => "rust",
        "ts" => "typescript",
        "tsx" => "typescriptreact",
        "js" => "javascript",
        "jsx" => "javascriptreact",
        "py" | "pyi" => "python",
        "go" => "go",
        "rb" => "ruby",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "scala" | "sc" => "scala",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
        "cs" => "csharp",
        "swift" => "swift",
        "sh" | "bash" | "zsh" => "shellscript",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "md" => "markdown",
        "html" | "htm" => "html",
        "css" | "scss" | "sass" | "less" => "css",
        "sql" => "sql",
        "lua" => "lua",
        "r" => "r",
        "dart" => "dart",
        "ex" | "exs" => "elixir",
        "erl" => "erlang",
        "hs" => "haskell",
        "zig" => "zig",
        "sol" => "solidity",
        "vue" => "vue",
        "svelte" => "svelte",
        "php" => "php",
        "nix" => "nix",
        _ => "plaintext",
    }
}

fn lsp_server_for_language(lang: &str) -> Option<Vec<String>> {
    match lang {
        "rust" => Some(vec!["rust-analyzer".into()]),
        "typescript" | "typescriptreact" | "javascript" | "javascriptreact" => {
            Some(vec!["typescript-language-server".into(), "--stdio".into()])
        }
        "python" => Some(vec!["pyright-langserver".into(), "--stdio".into()]),
        "go" => Some(vec!["gopls".into()]),
        _ => None,
    }
}

impl LspSession {
    pub fn new() -> Self {
        let ws = std::env::current_dir()
            .map(|p| format!("file://{}", p.display()))
            .unwrap_or_else(|_| "file:///".into());
        let workspace_uri = ws.parse().unwrap_or_else(|_| "file:///".parse().expect("static uri"));
        Self { process: None, stdin: None, reader: None, next_id: 1, initialized: false, workspace_uri }
    }

    async fn ensure_initialized_for(&mut self, file_path: &str) -> anyhow::Result<()> {
        if self.initialized {
            return Ok(());
        }
        let lang = detect_language(file_path);
        let server_cmd = lsp_server_for_language(lang);
        let cmd_args = server_cmd.unwrap_or_else(|| vec!["rust-analyzer".into()]);
        let (binary, args) = if cmd_args.len() > 1 {
            (cmd_args[0].clone(), cmd_args[1..].to_vec())
        } else {
            (cmd_args.into_iter().next().unwrap_or_else(|| "rust-analyzer".into()), Vec::new())
        };

        let init = async {
            let mut child = Command::new(&binary)
                .args(&args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| anyhow::anyhow!("failed to start LSP server '{}': {}. Is it installed?", binary, e))?;

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
            anyhow::Ok(())
        };

        match tokio::time::timeout(std::time::Duration::from_secs(15), init).await {
            Ok(Ok(())) => {
                self.initialized = true;
                crate::runtime::mark_lsp_initialized();
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                if let Some(mut c) = self.process.take() { let _ = c.start_kill(); }
                self.stdin = None;
                self.reader = None;
                Err(anyhow::anyhow!("LSP server '{}' initialization timed out", binary))
            }
        }
    }

    async fn ensure_initialized(&mut self) -> anyhow::Result<()> {
        if self.initialized {
            return Ok(());
        }
        self.ensure_initialized_for("main.rs").await
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub async fn get_diagnostics(&mut self, file_path: &str) -> anyhow::Result<Vec<String>> {
        self.ensure_initialized_for(file_path).await?;
        self.open_document(file_path).await?;
        let target_uri = path_to_uri(file_path)?;

        loop {
            let read = crate::protocol::transport::read_content_length_message(self.reader.as_mut().expect("LSP reader not initialized"), "LSP");
            match tokio::time::timeout(LSP_DIAGNOSTICS_TIMEOUT, read).await {
                Ok(Ok(msg)) => {
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
                Ok(Err(e)) => return Err(e),
                Err(_) => return Ok(Vec::new()),
            }
        }
    }

    pub async fn get_definition(&mut self, file_path: &str, line: u32, character: u32) -> anyhow::Result<String> {
        self.ensure_initialized_for(file_path).await?;
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
        self.ensure_initialized_for(file_path).await?;
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

    pub async fn get_hover(&mut self, file_path: &str, line: u32, character: u32) -> anyhow::Result<String> {
        self.ensure_initialized_for(file_path).await?;
        self.open_document(file_path).await?;
        let uri = path_to_uri(file_path)?;

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result: Option<Hover> = self.send_request("textDocument/hover", params).await?;
        match result {
            None => Ok("No hover information available".into()),
            Some(hover) => {
                let content = match hover.contents {
                    HoverContents::Scalar(MarkedString::String(s)) => s,
                    HoverContents::Scalar(MarkedString::LanguageString(ls)) => {
                        format!("[{}] {}", ls.language, ls.value)
                    }
                    HoverContents::Array(items) => {
                        items.iter().map(|item| match item {
                            MarkedString::String(s) => s.clone(),
                            MarkedString::LanguageString(ls) => format!("[{}] {}", ls.language, ls.value),
                        }).collect::<Vec<_>>().join("\n---\n")
                    }
                    HoverContents::Markup(markup) => markup.value,
                };
                let range_info = hover.range.map(|r| {
                    format!("\nRange: L{}:C{} - L{}:C{}", r.start.line + 1, r.start.character + 1, r.end.line + 1, r.end.character + 1)
                }).unwrap_or_default();
                Ok(format!("{}{}", content, range_info))
            }
        }
    }

    pub async fn get_completion(&mut self, file_path: &str, line: u32, character: u32) -> anyhow::Result<String> {
        self.ensure_initialized_for(file_path).await?;
        self.open_document(file_path).await?;
        let uri = path_to_uri(file_path)?;

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line, character },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: Some(CompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
        };

        let result: Option<CompletionResponse> = self.send_request("textDocument/completion", params).await?;
        match result {
            None => Ok("No completions available".into()),
            Some(CompletionResponse::Array(items)) => {
                if items.is_empty() {
                    return Ok("No completions found".into());
                }
                let lines: Vec<String> = items.iter().map(|item| {
                    let detail = item.detail.as_deref().unwrap_or("");
                    format!("  {} {} {}", item.insert_text.as_deref().unwrap_or(&item.label), detail, item.filter_text.as_deref().unwrap_or(""))
                }).collect();
                Ok(format!("Completions ({}):\n{}", lines.len(), lines.join("\n")))
            }
            Some(CompletionResponse::List(list)) => {
                if list.items.is_empty() {
                    return Ok("No completions found".into());
                }
                let lines: Vec<String> = list.items.iter().map(|item| {
                    let detail = item.detail.as_deref().unwrap_or("");
                    format!("  {} {}", item.label, detail)
                }).collect();
                Ok(format!("Completions ({}):\n{}", lines.len(), lines.join("\n")))
            }
        }
    }

    pub async fn get_rename(&mut self, file_path: &str, line: u32, character: u32, new_name: &str) -> anyhow::Result<String> {
        self.ensure_initialized_for(file_path).await?;
        self.open_document(file_path).await?;
        let uri = path_to_uri(file_path)?;

        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line, character },
            },
            new_name: new_name.into(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result: Option<WorkspaceEdit> = self.send_request("textDocument/rename", params).await?;
        match result {
            None => Ok("Rename not available at this position".into()),
            Some(edit) => {
                let mut changes = Vec::new();
                if let Some(edits) = &edit.document_changes {
                    match edits {
                        DocumentChanges::Edits(doc_edits) => {
                            for e in doc_edits {
                                let uri_str = e.text_document.uri.to_string();
                                let count = e.edits.len();
                                changes.push(format!("{} ({} changes)", uri_str, count));
                            }
                        }
                        DocumentChanges::Operations(ops) => {
                            for op in ops {
                                if let DocumentChangeOperation::Edit(e) = op {
                                    let uri_str = e.text_document.uri.to_string();
                                    changes.push(format!("{} ({} edits)", uri_str, e.edits.len()));
                                }
                            }
                        }
                    }
                } else if let Some(map) = &edit.changes {
                    for (uri, edits) in map {
                        #[allow(clippy::to_string_in_format_args)]
                        changes.push(format!("{} ({} edits)", uri.to_string(), edits.len()));
                    }
                }
                if changes.is_empty() {
                    Ok("No changes needed for rename".into())
                } else {
                    Ok(format!("Rename '{}':\n{}", new_name, changes.join("\n")))
                }
            }
        }
    }

    pub async fn get_document_symbols(&mut self, file_path: &str) -> anyhow::Result<String> {
        self.ensure_initialized_for(file_path).await?;
        self.open_document(file_path).await?;
        let uri = path_to_uri(file_path)?;

        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result: Option<DocumentSymbolResponse> = self.send_request("textDocument/documentSymbol", params).await?;
        match result {
            None => Ok("No symbols found".into()),
            Some(DocumentSymbolResponse::Flat(symbols)) => {
                let lines: Vec<String> = symbols.iter().map(|s| {
                    let icon = symbol_kind_icon(s.kind);
                    let uri_str = s.location.uri.to_string();
                    format!("  {} {}: {}", icon, s.name, uri_str)
                }).collect();
                Ok(format!("Symbols ({}):\n{}", lines.len(), lines.join("\n")))
            }
            Some(DocumentSymbolResponse::Nested(symbols)) => {
                let mut lines = Vec::new();
                for s in symbols {
                    format_symbol_tree(&s, 0, &mut lines);
                }
                Ok(format!("Symbols:\n{}", lines.join("\n")))
            }
        }
    }

    async fn open_document(&mut self, file_path: &str) -> anyhow::Result<()> {
        let uri = path_to_uri(file_path)?;
        let content = tokio::fs::read_to_string(file_path).await?;
        let language_id = detect_language(file_path);
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

        loop {
            let read = crate::protocol::transport::read_content_length_message(reader, "LSP");
            match tokio::time::timeout(LSP_REQUEST_TIMEOUT, read).await {
                Ok(Ok(msg)) => {
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
                Ok(Err(e)) => return Err(e),
                Err(_) => anyhow::bail!("LSP request timed out (id={})", id),
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

fn symbol_kind_icon(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::FILE => "",
        SymbolKind::MODULE => "[M]",
        SymbolKind::NAMESPACE => "[N]",
        SymbolKind::PACKAGE => "[P]",
        SymbolKind::CLASS => "[C]",
        SymbolKind::METHOD => "[m]",
        SymbolKind::PROPERTY => "[p]",
        SymbolKind::FIELD => "[f]",
        SymbolKind::CONSTRUCTOR => "[c]",
        SymbolKind::ENUM => "[E]",
        SymbolKind::INTERFACE => "[I]",
        SymbolKind::FUNCTION => "[fn]",
        SymbolKind::VARIABLE => "[v]",
        SymbolKind::CONSTANT => "[const]",
        SymbolKind::STRING => "[str]",
        SymbolKind::NUMBER => "[num]",
        SymbolKind::BOOLEAN => "[bool]",
        SymbolKind::ARRAY => "[arr]",
        SymbolKind::OBJECT => "[obj]",
        SymbolKind::KEY => "[K]",
        SymbolKind::NULL => "[null]",
        SymbolKind::ENUM_MEMBER => "[e]",
        SymbolKind::STRUCT => "[S]",
        SymbolKind::EVENT => "[evt]",
        SymbolKind::OPERATOR => "[op]",
        SymbolKind::TYPE_PARAMETER => "[T]",
        _ => "[?]",
    }
}

fn format_symbol_tree(sym: &DocumentSymbol, depth: usize, lines: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    let icon = symbol_kind_icon(sym.kind);
    let detail = sym.detail.as_deref().unwrap_or("");
    lines.push(format!("{}{} L{}: {} {}", indent, icon, sym.range.start.line + 1, sym.name, detail));
    for child in sym.children.iter().flatten() {
        format_symbol_tree(child, depth + 1, lines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language_rust() {
        assert_eq!(detect_language("main.rs"), "rust");
        assert_eq!(detect_language("lib.rs"), "rust");
    }

    #[test]
    fn test_detect_language_typescript() {
        assert_eq!(detect_language("app.ts"), "typescript");
        assert_eq!(detect_language("component.tsx"), "typescriptreact");
    }

    #[test]
    fn test_detect_language_javascript() {
        assert_eq!(detect_language("index.js"), "javascript");
        assert_eq!(detect_language("app.jsx"), "javascriptreact");
    }

    #[test]
    fn test_detect_language_python() {
        assert_eq!(detect_language("main.py"), "python");
        assert_eq!(detect_language("util.pyi"), "python");
    }

    #[test]
    fn test_detect_language_go() {
        assert_eq!(detect_language("server.go"), "go");
    }

    #[test]
    fn test_detect_language_markdown() {
        assert_eq!(detect_language("README.md"), "markdown");
    }

    #[test]
    fn test_detect_language_unknown() {
        assert_eq!(detect_language("Makefile"), "plaintext");
        assert_eq!(detect_language(""), "plaintext");
    }

    #[test]
    fn test_symbol_kind_icons() {
        assert_eq!(symbol_kind_icon(SymbolKind::FILE), "");
        assert_eq!(symbol_kind_icon(SymbolKind::MODULE), "[M]");
        assert_eq!(symbol_kind_icon(SymbolKind::FUNCTION), "[fn]");
        assert_eq!(symbol_kind_icon(SymbolKind::STRUCT), "[S]");
        assert_eq!(symbol_kind_icon(SymbolKind::ENUM), "[E]");
        assert_eq!(symbol_kind_icon(SymbolKind::CLASS), "[C]");
    }

    #[test]
    fn test_symbol_kind_icon_unknown() {
        assert_eq!(symbol_kind_icon(SymbolKind::TYPE_PARAMETER), "[T]");
    }

    #[test]
    fn test_format_symbol_tree_flat() {
        let sym = DocumentSymbol {
            name: "main".into(),
            kind: SymbolKind::FUNCTION,
            range: lsp_types::Range {
                start: lsp_types::Position { line: 0, character: 0 },
                end: lsp_types::Position { line: 10, character: 0 },
            },
            selection_range: lsp_types::Range {
                start: lsp_types::Position { line: 0, character: 0 },
                end: lsp_types::Position { line: 10, character: 0 },
            },
            detail: Some("fn main()".into()),
            children: None,
            tags: None,
            deprecated: None,
        };
        let mut lines = Vec::new();
        format_symbol_tree(&sym, 0, &mut lines);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("L1: main"));
    }

    #[test]
    fn test_format_symbol_tree_nested() {
        let child = DocumentSymbol {
            name: "inner_fn".into(),
            kind: SymbolKind::FUNCTION,
            range: lsp_types::Range {
                start: lsp_types::Position { line: 2, character: 0 },
                end: lsp_types::Position { line: 5, character: 0 },
            },
            selection_range: lsp_types::Range {
                start: lsp_types::Position { line: 2, character: 0 },
                end: lsp_types::Position { line: 5, character: 0 },
            },
            detail: None,
            children: None,
            tags: None,
            deprecated: None,
        };
        let parent = DocumentSymbol {
            name: "mod".into(),
            kind: SymbolKind::MODULE,
            range: lsp_types::Range {
                start: lsp_types::Position { line: 0, character: 0 },
                end: lsp_types::Position { line: 10, character: 0 },
            },
            selection_range: lsp_types::Range {
                start: lsp_types::Position { line: 0, character: 0 },
                end: lsp_types::Position { line: 10, character: 0 },
            },
            detail: None,
            children: Some(vec![child]),
            tags: None,
            deprecated: None,
        };
        let mut lines = Vec::new();
        format_symbol_tree(&parent, 0, &mut lines);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("[M]"));
        assert!(lines[0].contains("L1: mod"));
        assert!(lines[1].contains("L3: inner_fn"));
    }

    #[test]
    fn test_format_location_basic() {
        let loc = Location {
            uri: "file:///src/main.rs".parse().unwrap(),
            range: lsp_types::Range {
                start: lsp_types::Position { line: 4, character: 0 },
                end: lsp_types::Position { line: 4, character: 10 },
            },
        };
        let result = format_location(&loc);
        assert!(result.contains("5:1-5:11"));
        assert!(result.contains("main.rs"));
    }
}
