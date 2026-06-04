pub mod ansi;
pub mod builders;
pub mod components;
pub mod markdown;
pub mod scroller;

use ratatui::{Frame, layout::Rect};
use ratatui::style::Style;

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

    let theme = &app.config.theme;
    let selected = if app.overlay.selection_mode { app.overlay.selected_message } else { None };

    let buf = f.buffer_mut();
    scr.render(&components, area, buf, theme, selected);

    // Top border
    let at_bottom = scr.scroll >= scr.max_scroll();
    let border_color = if at_bottom { theme.dim_text() } else { theme.primary() };
    let border_style = Style::default().fg(border_color);
    for x in area.left()..area.right() {
        if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(x, area.y)) {
            cell.set_char('─').set_style(border_style);
        }
    }

    // Store offsets + chat area y for mouse click dispatch
    app.component_offsets = scr.offsets().to_vec();
    app.component_total_height = scr.total() as usize;
    app.chat_y = area.y;
    app.max_scroll = scr.max_scroll() as usize;
    app.scroll_lines = scr.scroll as usize;
}
