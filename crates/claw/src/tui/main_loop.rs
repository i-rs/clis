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
    const REMINDER_INTERVAL_SECS: u64 = 120;

    loop {
        terminal.draw(|f| crate::ui::render(f, app))?;

        // Process LLM events
        while let Ok(event) = llm_rx.try_recv() {
            let mut handler = LlmEventHandler::new(app, app_core);
            if matches!(handler.handle(event), Action::Quit) {
                break;
            }
        }

        // Periodic background reminder check (every 2 minutes, non-blocking)
        let elapsed = last_reminder_check.elapsed().as_secs();
        if elapsed >= REMINDER_INTERVAL_SECS && !app.is_processing() {
            let h = rt.spawn_blocking(reminders::check_reminders);
            if let Ok(Some(reminder_text)) = rt.block_on(h) {
                app.reminder_text = Some(reminder_text);
            }
            last_reminder_check = Instant::now();
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
