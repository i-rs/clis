use crate::agent::event::AgentEvent;
use crate::app::{AgentMessage, App, AppMode, ToolCallInfo};

pub async fn handle_event(event: AgentEvent, app: &mut App) {
    app.needs_redraw = true;
    match event {
        AgentEvent::Token(t) => {
            if let Some(ref mut s) = app.streaming {
                s.content.push_str(&t);
            }
        }
        AgentEvent::Reasoning(r) => {
            if let Some(ref mut s) = app.streaming {
                s.reasoning.push_str(&r);
            }
        }
        AgentEvent::ToolCallStart {
            id: _id,
            name,
            args,
        } => {
            let path_key = args
                .get("file_path")
                .or_else(|| args.get("path"))
                .and_then(|v| v.as_str());
            if matches!(name.as_str(), "write" | "edit" | "delete")
                && let Some(path) = path_key
            {
                let rel = super::super::make_relative(&app.current_dir, path);
                app.file_changes.insert(rel.clone());
                if !matches!(name.as_str(), "delete")
                    && let Ok(content) = tokio::fs::read_to_string(path).await
                {
                    app.last_file_states.push((rel, content));
                }
            }
            if name.as_str() == "rename"
                && let Some(from) = args.get("from").and_then(|v| v.as_str())
            {
                app.file_changes
                    .insert(super::super::make_relative(&app.current_dir, from));
                if let Some(to) = args.get("to").and_then(|v| v.as_str()) {
                    app.file_changes.insert(super::super::make_relative(&app.current_dir, to));
                }
            }
            let diff = if name.as_str() == "edit" {
                super::super::compute_diff_from_args(&args)
            } else {
                None
            };
            let info = ToolCallInfo {
                name,
                args: serde_json::to_string_pretty(&args).unwrap_or_default(),
                result: None,
                diff,
                duration_ms: 0,
            };
            if let Some(ref mut s) = app.streaming {
                s.tool_start = std::time::Instant::now();
                s.current_tool = Some(info);
            }
        }
        AgentEvent::ToolCallEnd {
            id: _id,
            name,
            result,
        } => {
            let mut finished_tool = None;
            let mut tool_step = 0;
            if let Some(ref mut s) = app.streaming
                && let Some(mut tool) = s.current_tool.take()
            {
                s.tool_counter += 1;
                tool_step = s.tool_counter;
                tool.result = Some(result.clone());
                tool.duration_ms = s.tool_start.elapsed().as_millis() as u64;
                finished_tool = Some(tool);
            }

            if let Some(tool) = finished_tool {
                let preview: String = result.chars().take(2000).collect();
                let display = if preview.len() < result.len() {
                    format!("{}\n{}...", name, preview)
                } else {
                    format!("{}\n{}", name, preview)
                };
                app.push_message(AgentMessage::ToolResult {
                    content: format!("\n{}", display),
                    diff: tool.diff.clone(),
                    step: tool_step,
                    total_steps: 0,
                    collapsed: true,
                    duration_ms: tool.duration_ms,
                });

                if let Some(ref mut s) = app.streaming {
                    s.tool_calls.push(tool);
                }
            }
        }
        AgentEvent::Status(msg) => {
            app.status_message = Some(msg);
        }
        AgentEvent::Plan { steps } => {
            app.plan = steps;
        }
        AgentEvent::Done {
            usage,
            messages,
            context_pct,
        } => {
            app.status_message = None;
            let total_duration_ms = app
                .streaming
                .as_ref()
                .map(|s| s.start_time.elapsed().as_millis() as u64)
                .unwrap_or(0);
            let streamed_tc = app
                .streaming
                .as_ref()
                .map(|s| s.tool_calls.clone())
                .unwrap_or_default();
            let (content, reasoning) = app.finish_streaming();

            let total_tools = streamed_tc.len();
            if total_tools > 1 {
                for msg in app.messages.iter_mut().rev() {
                    if let AgentMessage::ToolResult {
                        total_steps, ..
                    } = msg
                    {
                        *total_steps = total_tools;
                    } else {
                        break;
                    }
                }
            }

            if !messages.is_empty() {
                app.agent_messages = messages.clone();
            }
            if let Some(u) = usage {
                app.add_token_usage(u.input_tokens, u.output_tokens);
            }
            app.context_usage = Some(context_pct);
            let tool_calls = super::super::extract_tool_calls(&messages);
            if !content.is_empty() || !reasoning.is_empty() || tool_calls.is_some() {
                app.push_message(AgentMessage::Assistant {
                    content,
                    reasoning,
                    tool_calls,
                    reasoning_expanded: false,
                    duration_ms: total_duration_ms,
                });
            }

            if !app.file_changes.is_empty() {
                let mut files: Vec<&String> = app.file_changes.iter().collect();
                files.sort();
                let mut summary = format!("── 完成 ──\n📄 {} 个文件:", files.len());
                for f in &files {
                    summary.push_str(&format!("\n  {}", f));
                }
                if !streamed_tc.is_empty() {
                    let mut tool_counts: std::collections::BTreeMap<&str, usize> =
                        std::collections::BTreeMap::new();
                    for tc in &streamed_tc {
                        *tool_counts.entry(tc.name.as_str()).or_insert(0) += 1;
                    }
                    let tool_str: Vec<String> = tool_counts
                        .iter()
                        .map(|(n, c)| format!("{} ×{}", n, c))
                        .collect();
                    summary.push_str(&format!("\n🔧 {}", tool_str.join("  ")));
                }
                app.push_message(AgentMessage::system(summary));
            }

            app.push_message(AgentMessage::Separator {
                label: String::new(),
            });

            if matches!(app.mode, AppMode::Waiting) {
                app.mode = AppMode::Idle;
            }
        }
        AgentEvent::Error(e) => {
            app.status_message = None;
            let (content, reasoning) = app.finish_streaming();
            let msg = if !content.is_empty() {
                format!("{}\n\nError: {}", content, e)
            } else {
                format!("Error: {}", e)
            };
            app.push_message(AgentMessage::Assistant {
                content: msg,
                reasoning,
                tool_calls: None,
            reasoning_expanded: false,
            duration_ms: 0,
        });
        if matches!(app.mode, AppMode::Waiting) {
                app.mode = AppMode::Idle;
            }
        }
    }
}
