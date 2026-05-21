use crate::app;
use crate::core;
use crate::llm::LlmEvent;
use crate::tui::event_handlers::{Action, KeyEventHandler, LlmEventHandler, MouseEventHandler};
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
    const REMINDER_INTERVAL_SECS: u64 = 120;
    const MCP_HEALTH_INTERVAL_SECS: u64 = 300;

    loop {
        terminal.draw(|f| crate::ui::render(f, app))?;

        while let Ok(event) = llm_rx.try_recv() {
            let mut handler = LlmEventHandler::new(app, app_core);
            if matches!(handler.handle(event), Action::Quit) {
                break;
            }
        }

        if last_reminder_check.elapsed().as_secs() >= REMINDER_INTERVAL_SECS && !app.is_processing() {
            let h = rt.spawn_blocking(reminders::check_reminders);
            if let Ok(Some(reminder_text)) = rt.block_on(h) {
                app.reminder_text = Some(reminder_text);
            }
            last_reminder_check = Instant::now();
        }

        if last_mcp_health_check.elapsed().as_secs() >= MCP_HEALTH_INTERVAL_SECS && !app.is_processing() {
            let mcp = app_core.agent_store.mcp_registry_for_mut(&app.current_agent);
            let reconnected = mcp.health_check_and_reconnect();
            if reconnected > 0 {
                tracing::info!("MCP 健康检查: {} 个客户端已重连", reconnected);
            }
            last_mcp_health_check = Instant::now();
        }

        // Handle terminal events
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    let mut kh = KeyEventHandler::new(app, app_core, rt, llm_tx);
                    if matches!(kh.handle(key), Action::Quit) {
                        break;
                    }
                }
                Event::Mouse(mouse) => {
                    MouseEventHandler::new(app).handle(mouse);
                }
                _ => {}
            }
        }
    }

    Ok(())
}
