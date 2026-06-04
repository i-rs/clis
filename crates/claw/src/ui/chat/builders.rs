//! Small line-construction helpers shared by the chat renderers.
//!
//! Historically this module also held the per-message `build_*_lines`
//! functions used by the global `build_components` helper. Those
//! helpers were removed when message rendering moved into the
//! per-type component modules under `super::components::*` — their
//! expand/collapse state used to be looked up out of `OverlayState`
//! hash sets, but state now lives on the component itself. The
//! remaining `indent_line` / `padded_line` / `timestamp_label`
//! helpers are still consumed by `super::ansi` and a couple of the
//! component renderers, so they stay.

use ratatui::{
    style::Style,
    text::{Line, Span},
};

use crate::app::App;
use crate::ui::utils;

pub(super) fn timestamp_label(app: &App, msg_index: usize, now: chrono::NaiveDateTime) -> String {
    let now_ts = now.and_utc().timestamp();
    app.message_timestamps
        .get(msg_index)
        .map(|ts| {
            format!(
                "  [{}]",
                utils::relative_time_at(ts.and_utc().timestamp(), now_ts)
            )
        })
        .unwrap_or_default()
}

pub(super) fn indent_line(text: &str, style: Style) -> Line<'static> {
    Line::from(vec![Span::raw("   "), Span::styled(text.to_string(), style)])
}

pub(super) fn padded_line(text: &str, style: Style) -> Line<'static> {
    Line::from(Span::styled(text.to_string(), style))
}
