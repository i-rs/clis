mod app;
mod config;
mod llm;
mod tools;
mod ui;

use crate::config::Config;
use crate::llm::LlmEvent;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tokio::sync::mpsc;

fn main() -> anyhow::Result<()> {
    let config = Config::load()?;

    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let rt = tokio::runtime::Runtime::new()?;
    let (llm_tx, mut llm_rx) = mpsc::unbounded_channel::<LlmEvent>();

    let mut app = app::App::new(config);

    // Welcome message
    app.messages.push(app::Message::Assistant {
        text: "你好！我是 i-rs-claw，你的个人数据智能助理。\
               \n我可以帮你管理健康、财务、任务、媒体等个人信息。\
               \n试试说：\"记录体重75kg\" 或 \"最近跑步情况如何？\""
            .to_string(),
    });

    'main_loop: loop {
        terminal.draw(|f| ui::render(f, &app))?;

        // Process LLM events
        while let Ok(event) = llm_rx.try_recv() {
            match event {
                LlmEvent::NewRound => {
                    app.start_assistant_message();
                }
                LlmEvent::Token(text) => {
                    app.append_assistant_text(&text);
                }
                LlmEvent::Status(text) => {
                    app.set_status(&text);
                }
                LlmEvent::ToolExecuted {
                    name,
                    args,
                    result,
                } => {
                    app.add_tool_call(&name, &args, &result);
                }
                LlmEvent::Error(text) => {
                    app.add_error(&text);
                }
                LlmEvent::Done(msgs) => {
                    app.finish_processing(Some(msgs));
                }
            }
        }

        // Handle terminal events
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') if key.modifiers == KeyModifiers::CONTROL => {
                        break 'main_loop;
                    }
                    KeyCode::Enter => {
                        if !app.input.is_empty() && !app.is_processing() {
                            let text = std::mem::take(&mut app.input);
                            app.add_user_message(&text);

                            // Build messages for LLM (preserve context from saved API messages)
                            let msgs = crate::llm::build_messages(
                                &app.messages,
                                &text,
                                &app.api_messages,
                            );

                            // Spawn LLM chat in background
                            let config = app.config.clone();
                            let tx = llm_tx.clone();
                            rt.spawn(async move {
                                crate::llm::chat_loop(config, msgs, tx).await;
                            });
                        }
                    }
                    KeyCode::Backspace => {
                        if !app.input.is_empty() {
                            app.input.pop();
                        }
                    }
                    KeyCode::Char(c) => {
                        if !app.is_processing() {
                            app.input.push(c);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    Ok(())
}
