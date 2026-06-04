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

    // If the component cache is out of sync (e.g. session load, or
    // any path that mutated `messages` without mirroring to
    // `components`), rebuild it. `push_component_for` keeps them in
    // lock-step for the streaming hot path so this is normally a no-op.
    if app.components.len() != app.messages.len() {
        app.rebuild_components();
    }

    let mut scr = scroller::Scroller::new(&app.components, width, viewport_h);
    scr.set_scroll(app.scroll_lines as u16);

    let theme = &app.config.theme;
    let selected = if app.overlay.selection_mode { app.overlay.selected_message } else { None };

    let buf = f.buffer_mut();
    scr.render(&app.components, area, buf, theme, selected);

    // Top border
    let at_bottom = scr.scroll >= scr.max_scroll();
    let border_color = if at_bottom { theme.dim_text() } else { theme.primary() };
    let border_style = Style::default().fg(border_color);
    for x in area.left()..area.right() {
        if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(x, area.y)) {
            cell.set_char('─').set_style(border_style);
        }
    }

    // Hand the scroller's content-relative click regions to the
    // library's registry, translated to screen-absolute coordinates.
    // The registry now owns the click dispatch — the input handlers
    // just need the absolute row/col of the click event.
    scr.register_clicks(area, &mut app.hit_regions);
    app.chat_y = area.y;
    app.max_scroll = scr.max_scroll() as usize;
    app.scroll_lines = scr.scroll as usize;
}
