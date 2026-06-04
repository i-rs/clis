pub mod ansi;
pub mod builders;
pub mod components;
pub mod markdown;
pub mod scroller;

use ratatui::{Frame, layout::Rect};
use ratatui::style::Style;

use crate::app::App;
use crate::config::Config;

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

    let theme = &app.config.theme;
    let selected = if app.overlay.selection_mode { app.overlay.selected_message } else { None };

    let buf = f.buffer_mut();

    // Render visible components
    scr.render(&components, area, buf, theme, selected);

    // Draw top border
    let at_bottom = scr.scroll >= scr.max_scroll();
    let border_color = if at_bottom { theme.dim_text() } else { theme.primary() };
    let border_style = Style::default().fg(border_color);
    for x in area.left()..area.right() {
        if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(x, area.y)) {
            cell.set_char('─').set_style(border_style);
        }
    }

    // Store component y-offsets for mouse click dispatch
    let mut offsets = Vec::with_capacity(components.len());
    let mut total = 0u16;
    for c in &components {
        offsets.push(total);
        total += c.height(width);
    }
    app.component_offsets = offsets;
    app.component_total_height = total as usize;
    app.max_scroll = scr.max_scroll() as usize;
    app.scroll_lines = scr.scroll as usize;
}
