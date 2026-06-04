//! Shared visual helpers for message components.
//!
//! All message blocks share the same aesthetic:
//!
//! ```text
//!     ╭─ ── ── ── ── ── ── ── ── ──╮     <- top rounded border
//!     ▎  Avatar  Label   timestamp        <- header line
//!     ▎  body line 1
//!     ▎  body line 2
//!     ╰─ ── ── ── ── ── ── ── ── ──╯     <- bottom rounded border
//! ```
//!
//! The avatar is a single rounded cell in the left margin that visually
//! anchors the block. Header, body and footer share the same left margin
//! so the block reads as a single, self-contained unit.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::theme::Theme;

/// Inner horizontal padding (in cells) on the left side of every block.
pub const BLOCK_INDENT: usize = 2;

/// Width (in cells) of the left "avatar gutter" used to anchor the block.
pub const BLOCK_GUTTER: usize = 3;

/// Total columns reserved on the left for the avatar gutter.
/// Body content may use everything from here to the right edge.
pub const BLOCK_LEFT_RESERVED: usize = BLOCK_INDENT + BLOCK_GUTTER;

/// Build the top rounded border for a block of the given total width.
///
/// ```text
/// ╭───────────╮
/// ```
pub fn rounded_top(width: u16, border: Color) -> Line<'static> {
    if width < 2 {
        return Line::from("");
    }
    let inner = (width as usize).saturating_sub(2);
    Line::from(Span::styled(
        format!("╭{}╮", "─".repeat(inner)),
        Style::default().fg(border),
    ))
}

/// Build the bottom rounded border for a block of the given total width.
///
/// ```text
/// ╰───────────╯
/// ```
pub fn rounded_bottom(width: u16, border: Color) -> Line<'static> {
    if width < 2 {
        return Line::from("");
    }
    let inner = (width as usize).saturating_sub(2);
    Line::from(Span::styled(
        format!("╰{}╯", "─".repeat(inner)),
        Style::default().fg(border),
    ))
}

/// A header line: `<indent><glyph> <label><meta?>`.
///
/// `glyph` is the avatar/symbol in the gutter; `label` is the role
/// (`You`, `Claw`, tool name, ...). `meta` is an optional right-aligned
/// hint (e.g. `2分钟前`).
pub fn header_line(
    label: &str,
    glyph: &str,
    glyph_color: Color,
    label_color: Color,
    meta: Option<&str>,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(5);
    spans.push(Span::raw(" ".repeat(BLOCK_INDENT)));
    spans.push(Span::styled(
        glyph.to_string(),
        Style::default().fg(glyph_color).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        label.to_string(),
        Style::default().fg(label_color).add_modifier(Modifier::BOLD),
    ));
    if let Some(m) = meta {
        spans.push(Span::styled(
            format!("  {}", m),
            Style::default().fg(Color::Rgb(110, 110, 130)),
        ));
    }
    Line::from(spans)
}

/// A body line that lives inside a message block.
///
/// Indents the text by `BLOCK_LEFT_RESERVED` columns so it aligns with the
/// header label, and paints the gutter bar.
pub fn body_line(text: &str, style: Style) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(3);
    spans.push(Span::raw(" ".repeat(BLOCK_INDENT)));
    spans.push(Span::raw("▎ "));
    spans.push(Span::styled(text.to_string(), style));
    Line::from(spans)
}

/// A blank line that fills the gutter (no bar). Useful as a final
/// visual breath before the bottom border.
pub fn body_padding() -> Line<'static> {
    Line::from("")
}

/// Compute the border color a block should use, depending on selection
/// state. Selected blocks get a brighter border so they pop.
pub fn block_border(theme: &Theme, selected: bool) -> Color {
    if selected {
        theme.primary()
    } else {
        theme.border()
    }
}

/// Per-block tint identifier. Reserved for future use.
#[allow(dead_code)]
pub enum BlockTint {
    User,
    Assistant,
    Tool,
    Error,
    Evaluation,
    Quality,
    Info,
}

/// Mix two colors. `t` is 0.0..=1.0 — at 0 returns `a`, at 1 returns `b`.
/// Used to brighten a block's surface when it is selected.
pub fn blend(a: Color, b: Color, t: f32) -> Color {
    let (ar, ag, ab) = split(a);
    let (br, bg, bb) = split(b);
    let mix = |x: u8, y: u8| -> u8 {
        let xv = x as f32;
        let yv = y as f32;
        (xv + (yv - xv) * t).round().clamp(0.0, 255.0) as u8
    };
    Color::Rgb(mix(ar, br), mix(ag, bg), mix(ab, bb))
}

fn split(c: Color) -> (u8, u8, u8) {
    if let Color::Rgb(r, g, b) = c {
        (r, g, b)
    } else {
        (128, 128, 128)
    }
}

/// The interior body region of a block — the slice of rows between
/// the header and the bottom border that the caller must fill in.
///
/// `top` is the first body row, `bottom` is the first row that is
/// *not* body (i.e. the bottom border or out-of-area). Body height
/// is therefore `bottom.saturating_sub(top)`.
#[derive(Clone, Copy, Debug)]
pub struct BodyArea {
    pub top: u16,
    pub bottom: u16,
}

impl BodyArea {
    /// Number of rows available for body content.
    pub fn height(&self) -> u16 {
        self.bottom.saturating_sub(self.top)
    }

    /// True if `y` falls inside the body region. Convenience for
    /// sequential renderers that walk rows top-down.
    pub fn contains(&self, y: u16) -> bool {
        y >= self.top && y < self.bottom
    }
}

/// Render the shared block chrome — top rounded border, header line,
/// bottom rounded border — and return the [`BodyArea`] the caller is
/// responsible for filling in.
///
/// Every message component uses the same outer shape; this helper
/// consolidates ~10 lines of `Paragraph::new(...).render(...)` per
/// component into a single call. The caller still chooses colours,
/// avatar glyph, label, meta (timestamp) and interior background.
///
/// Behaviour for degenerate `area.height < 3` matches the previous
/// per-component implementation: bottom border may overlap top/header.
pub fn render_block_chrome(
    area: Rect,
    buf: &mut Buffer,
    border: Color,
    interior_bg: Color,
    header: Line<'static>,
) -> BodyArea {
    // Top border.
    Paragraph::new(rounded_top(area.width, border))
        .style(Style::default().bg(interior_bg))
        .render(Rect { y: area.y, height: 1, ..area }, buf);

    // Header.
    Paragraph::new(header)
        .style(Style::default().bg(interior_bg))
        .render(
            Rect {
                y: area.y + 1,
                height: 1,
                ..area
            },
            buf,
        );

    // Bottom border. Always rendered at the last row, matching the
    // previous code's `if area.height >= 1` guard.
    if area.height >= 1 {
        let by = area.y + area.height - 1;
        Paragraph::new(rounded_bottom(area.width, border))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y: by, height: 1, ..area }, buf);
    }

    BodyArea {
        top: area.y + 2,
        bottom: area.y + area.height.saturating_sub(1),
    }
}
