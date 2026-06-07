use crate::app;
use i_rs_claw_core::core;
use i_rs_claw_core::llm::LlmEvent;
use crate::tui::handlers::{Action, KeyEventHandler, LlmEventHandler, MouseEventHandler};
use crossterm::event::{self, Event};
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::Instant;
use tokio::sync::mpsc;

use super::reminders;

pub fn main_loop(
    terminal: &mut ratatui::Terminal<CrosstermBackend<io::Stdout>>,
    rt: &tokio::runtime::Runtime,
    app: &mut app::App,
    app_core: &mut core::AppCore,
    llm_tx: &mpsc::UnboundedSender<LlmEvent>,
    llm_rx: &mut mpsc::UnboundedReceiver<LlmEvent>,
) -> anyhow::Result<()> {
    let mut last_reminder_check = Instant::now();
    let mut last_mcp_health_check = Instant::now();
    let mut reminder_handle: Option<std::thread::JoinHandle<Option<String>>> = None;
    const REMINDER_INTERVAL_SECS: u64 = 120;
    const MCP_HEALTH_INTERVAL_SECS: u64 = 300;
    /// Spinner 帧间隔。processing 期间即使无事件也至少每帧刷新一次。
    const SPINNER_TICK_MS: u128 = 80;

    'outer: loop {
        // 条件重绘：仅在 dirty 或 processing 节拍时刷新。
        let need_continuous = app.is_processing();
        let spinner_tick = need_continuous
            && app
                .render_state
                .last_drawn_at
                .map(|t| t.elapsed().as_millis() >= SPINNER_TICK_MS)
                .unwrap_or(true);
        if app.render_state.dirty || spinner_tick {
            terminal.draw(|f| crate::ui::render(f, app))?;
            app.render_state.mark_rendered();
        }

        while let Ok(event) = llm_rx.try_recv() {
            let mut handler = LlmEventHandler::new(app, app_core);
            if matches!(handler.handle(event), Action::Quit) {
                break 'outer;
            }
        }

        if let Some(ref h) = reminder_handle
            && h.is_finished()
        {
            let handle = reminder_handle.take().unwrap();
            if let Some(reminder_text) = handle.join().unwrap_or(None) {
                app.reminder_text = Some(reminder_text);
            }
        }

        if reminder_handle.is_none()
            && last_reminder_check.elapsed().as_secs() >= REMINDER_INTERVAL_SECS
            && !app.is_processing()
        {
            reminder_handle = Some(std::thread::spawn(reminders::check_reminders));
            last_reminder_check = Instant::now();
        }

        if last_mcp_health_check.elapsed().as_secs() >= MCP_HEALTH_INTERVAL_SECS
            && !app.is_processing()
        {
            let mcp = app_core
                .agent_store
                .mcp_registry_for_mut("default", &app.current_agent)?;
            let reconnected = mcp.health_check_and_reconnect();
            if reconnected > 0 {
                tracing::info!("MCP 健康检查: {} 个客户端已重连", reconnected);
            }
            last_mcp_health_check = Instant::now();
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    let mut kh = KeyEventHandler::new(app, app_core, rt, llm_tx);
                    if matches!(kh.handle(key), Action::Quit) {
                        break 'outer;
                    }
                }
                Event::Mouse(mouse) => {
                    MouseEventHandler::new(app).handle(mouse);
                }
                Event::Paste(text) if !app.is_processing() => {
                    if !text.is_empty() {
                        let dropped = app.input.insert_text_at_cursor(&text);
                        app.overlay.tab_completions.clear();
                        if app.overlay.slash_visible && !app.input.text.starts_with('/') {
                            app.overlay.slash_visible = false;
                            app.overlay.slash_index = 0;
                        }
                        if dropped > 0 {
                            app.overlay.copy_feedback = Some((
                                format!(
                                    "粘贴已截断 (超 {} KiB 上限)",
                                    crate::app::MAX_INPUT_LEN / 1024
                                ),
                                std::time::Instant::now(),
                            ));
                        }
                        app.mark_overlay_dirty();
                    }
                }
                Event::Resize(_, _) => {
                    app.mark_dirty();
                }
                Event::Paste(_) => {}
                _ => {}
            }
        }
    }

    if let Some(h) = reminder_handle {
        let _ = h.join();
    }

    Ok(())
}
