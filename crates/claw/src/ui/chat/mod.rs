pub mod components;
pub mod markdown;
pub mod scroller;

use ratatui::style::Style;
use ratatui::{Frame, layout::Rect};

use crate::app::App;

pub(super) fn render_chat(f: &mut Frame, area: Rect, app: &mut App) {
    let width = area.width;
    let viewport_h = area.height.saturating_sub(1).max(1);

    // If the component cache is out of sync (e.g. session load, or
    // any path that mutated `messages` without mirroring to
    // `components`), rebuild it. `push_component_for` keeps them in
    // lock-step for the streaming hot path so this is normally a no-op.
    if app.chat.components.len() != app.chat.messages.len() {
        app.rebuild_components();
    }

    let mut scr = scroller::Scroller::new(&app.chat.components, width, viewport_h);
    // App's `scroll_lines` uses the Scroller's own convention
    // (0 = top / oldest, max_scroll = bottom / newest). When
    // `stick_to_bottom` is engaged, override to the current max so the
    // viewport tracks the live tail without App having to know the
    // exact content height (which changes every time tokens stream in).
    let max = scr.max_scroll();
    let scroll = if app.scroll.stick_to_bottom {
        max
    } else {
        (app.scroll.scroll_lines as u16).min(max)
    };
    scr.set_scroll(scroll);

    let theme = &app.config.theme;
    let selected = if app.overlay.selection_mode {
        app.overlay.selected_message
    } else {
        None
    };

    // Render components one row below area.y so the '─' border
    // at area.y does not overwrite the top border of the first
    // component.
    let inner_area = Rect {
        y: area.y + 1,
        ..area
    };
    let buf = f.buffer_mut();
    scr.render(&app.chat.components, inner_area, buf, theme, selected);

    // Top border. In the new convention `scr.scroll == max_scroll`
    // means the viewport is at the bottom (newest), so the dim border
    // correctly signals "no more content below".
    let at_bottom = scr.scroll >= scr.max_scroll();
    let border_color = if at_bottom {
        theme.dim_text()
    } else {
        theme.primary()
    };
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
    scr.register_clicks(inner_area, &mut app.hit_regions);
    app.scroll.max_scroll = scr.max_scroll() as usize;
    // Only persist `scroll_lines` when not sticky — sticky mode means
    // "the renderer decides", so saving `scr.scroll` back would freeze
    // the viewport once content stopped growing.
    if !app.scroll.stick_to_bottom {
        app.scroll.scroll_lines = scr.scroll as usize;
    }

    // Write layout-critical metrics back so event handlers (selection
    // mode scroll, expand/collapse) can compute viewport math without
    // waiting for the next render. Previously these fields were only
    // populated in tests, leaving selection-mode scrolling operating on
    // stale `area_lines = 1` / `text_width = 20` placeholders.
    app.render_state.cached_width = width as usize;
    app.render_state.chat_height = viewport_h;
    app.render_state.heights = scr.heights().iter().map(|&h| h as usize).collect();
}
