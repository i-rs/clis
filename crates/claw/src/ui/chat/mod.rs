pub mod ansi;
pub mod builders;
pub mod components;
pub mod markdown;
pub mod scroller;

use ratatui::{Frame, layout::Rect};
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::app::App;

pub(super) fn render_chat(f: &mut Frame, area: Rect, app: &mut App) {
    let width = area.width;
    let viewport_h = area.height.saturating_sub(1).max(1);

    let components = components::build_components(
        &app.messages,
        &app.overlay.tool_call_expanded,
        &app.overlay.reasoning_expanded,
    );

    let mut scr = scroller::Scroller::new(&components, width, viewport_h);
    scr.set_scroll(app.scroll_lines as u16);

    let theme = app.config.theme.clone();
    let selected = if app.overlay.selection_mode { app.overlay.selected_message } else { None };

    // Render into frame
    let at_bottom = scr.scroll >= scr.max_scroll();
    let border_style = Style::default().fg(if at_bottom { theme.dim_text() } else { theme.primary() });

    // Render scrollbar + border first, then components
    // Build a buffer to render into via the scroller
    let buf = f.buffer_mut();

    // Render each visible component
    scr.render(&components, area, buf, &theme, selected);

    // Draw scrollbar
    let total = scr.total();
    let vh = scr.viewport_h;
    if total > vh && area.width > 0 {
        let bar_h = ((vh as f64 / total as f64) * vh as f64).max(1.0) as u16;
        let bar_y = ((scr.scroll as f64 / total as f64) * vh as f64) as u16;
        let bar_x = area.x + area.width.saturating_sub(1);
        let dim = theme.dim_text();
        for i in 0..vh {
            let ch = if i >= bar_y && i < bar_y + bar_h { '█' } else { '░' };
            if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(bar_x, area.y + i)) {
                cell.set_char(ch).set_style(Style::default().fg(dim));
            }
        }
    }

    // Draw top border
    for x in area.left()..area.right() {
        if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(x, area.y)) {
            cell.set_char('─').set_style(border_style);
        }
    }

    // Update app state
    app.max_scroll = scr.max_scroll() as usize;
    app.scroll_lines = scr.scroll as usize;
}
