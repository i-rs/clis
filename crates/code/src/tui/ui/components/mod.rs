//! Per-message-type rendering components with height caching.
//!
//! Each `AgentMessage` maps to a `MessageComponent` that owns its
//! content, caches its height per width, and renders on demand.
//! The `Scroller` uses pre-computed heights to render only
//! visible components — no full rebuild on every frame.

mod assistant;
mod scroller;
mod simple;
mod streaming;
mod tool_result;

use crate::app::{AgentMessage, StreamingState};
use crate::tui::colors::*;
pub use scroller::Scroller;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use std::cell::RefCell;
use std::rc::Rc;

/// Operations the App can apply to a component after construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentOp {
    /// Append streamed text delta to the assistant body.
    #[allow(dead_code)]
    AppendText(String),
    /// Append streamed reasoning delta.
    #[allow(dead_code)]
    AppendReasoning(String),
    /// Flip expand/collapse state.
    Toggle,
}

/// A component that renders as part of the scrollable chat view.
///
/// Each component caches its height per `(content_key, width)` and
/// renders only the visible portion (via `y_offset`).
pub trait MessageComponent {
    /// Height in terminal rows at the given width.
    fn height(&self, width: u16) -> u16;

    /// Render the component into `area` of `buf`.
    ///
    /// `y_offset` is the number of rows scrolled off the top of this
    /// component (for partially-visible components during scroll).
    /// `selected` indicates whether the message is currently selected.
    fn render(&self, area: Rect, buf: &mut Buffer, y_offset: u16, selected: bool);

    /// Whether clicking anywhere on this component should toggle it.
    fn clickable(&self) -> bool {
        false
    }

    /// Apply a runtime mutation (streaming text, toggle, etc.).
    fn apply(&mut self, _op: ComponentOp) {}

    /// Extra click targets within the component.
    /// Returns `(row_offset, height, op)` triples relative to the
    /// component's *virtual* top (row 0).
    fn extra_click_targets(&self, _width: u16) -> Vec<(u16, u16, ComponentOp)> {
        vec![]
    }
}

/// Shared, mutable component handle.
pub type ComponentCell = Rc<RefCell<Box<dyn MessageComponent>>>;

// ─── Component factory ─────────────────────────────────────────────────────

/// Build a `MessageComponent` for an `AgentMessage`.
pub fn build_component_for(msg: &AgentMessage) -> Box<dyn MessageComponent> {
    match msg {
        AgentMessage::Assistant {
            content, reasoning, ..
        } => {
            if content.is_empty() && reasoning.is_empty() {
                return Box::new(EmptyComponent);
            }
            Box::new(assistant::AssistantBlock::new(content, reasoning, false))
        }
        AgentMessage::User { content } => {
            if content.is_empty() {
                return Box::new(EmptyComponent);
            }
            Box::new(simple::UserBubble::new(content))
        }
        AgentMessage::ToolResult {
            content,
            diff,
            step,
            total_steps,
            collapsed,
            ..
        } => Box::new(tool_result::ToolResultCard::new(
            content,
            diff.as_deref(),
            *step,
            *total_steps,
            *collapsed,
        )),
        AgentMessage::FileEdit { path, summary } => {
            Box::new(simple::FileEditBlock::new(path, summary))
        }
        AgentMessage::System { content } => {
            if content.is_empty() {
                return Box::new(EmptyComponent);
            }
            Box::new(simple::SystemBanner::new(content))
        }
        AgentMessage::Separator { label } => Box::new(simple::SeparatorLine::new(label)),
    }
}

/// Build a `MessageComponent` for the streaming state.
pub fn build_streaming_component(s: &StreamingState) -> Box<dyn MessageComponent> {
    Box::new(streaming::StreamingBlock::new(s))
}

// ─── Zero-height no-op component ─────────────────────────────────────────

struct EmptyComponent;
impl MessageComponent for EmptyComponent {
    fn height(&self, _w: u16) -> u16 {
        0
    }
    fn render(&self, _area: Rect, _buf: &mut Buffer, _y_offset: u16, _selected: bool) {}
    fn apply(&mut self, _op: ComponentOp) {}
}

// ═══════════════════════════════════════════════════════════════════════════
//  Shared style helpers (ported from claw)
// ═══════════════════════════════════════════════════════════════════════════

/// Inner horizontal padding (in cells) on the left side of every block.
pub const BLOCK_INDENT: usize = 2;

/// Top rounded border: `╭───────╮`
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

/// Bottom rounded border: `╰───────╯`
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

/// Header line: `<indent><glyph> <label><meta?>`
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
        Style::default()
            .fg(glyph_color)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        label.to_string(),
        Style::default()
            .fg(label_color)
            .add_modifier(Modifier::BOLD),
    ));
    if let Some(m) = meta {
        spans.push(Span::styled(
            format!("  {}", m),
            Style::default().fg(c_muted()),
        ));
    }
    Line::from(spans)
}

/// A body line that lives inside a message block.
///
/// Indents the text by `BLOCK_INDENT + 3` columns so it aligns with the
/// header label, and paints the gutter bar.
pub fn body_line(text: &str, style: Style) -> Line<'static> {
    let spans: Vec<Span<'static>> = vec![
        Span::raw(" ".repeat(BLOCK_INDENT)),
        Span::styled(text.to_string(), style),
    ];
    Line::from(spans)
}

/// Compute border color based on selection.
pub fn block_border(selected: bool) -> Color {
    if selected { c_accent() } else { c_border() }
}

/// Blend two colors by `t` (0.0..=1.0).
pub fn blend(a: Color, b: Color, t: f32) -> Color {
    let (ar, ag, ab) = split_rgb(a);
    let (br, bg, bb) = split_rgb(b);
    let mix = |x: u8, y: u8| -> u8 {
        (x as f32 + (y as f32 - x as f32) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color::Rgb(mix(ar, br), mix(ag, bg), mix(ab, bb))
}

fn split_rgb(c: Color) -> (u8, u8, u8) {
    if let Color::Rgb(r, g, b) = c {
        (r, g, b)
    } else {
        (128, 128, 128)
    }
}

/// The interior body region between header and bottom border.
#[derive(Clone, Copy, Debug)]
pub struct BodyArea {
    pub top: u16,
    pub bottom: u16,
}

impl BodyArea {
    #[allow(dead_code)]
    pub fn height(&self) -> u16 {
        self.bottom.saturating_sub(self.top)
    }
    pub fn contains(&self, y: u16) -> bool {
        y >= self.top && y < self.bottom
    }
}

/// Render the shared block chrome — top border, header, bottom border —
/// and return the `BodyArea` the caller fills in.
///
/// When `y_offset >= 2` the top border + header are skipped
/// (partial-scroll rendering).
pub fn render_block_chrome(
    area: Rect,
    buf: &mut Buffer,
    border: Color,
    interior_bg: Color,
    header: Line<'static>,
    y_offset: u16,
) -> BodyArea {
    let bg_style = Style::default().bg(interior_bg);

    // Top border (skip if scrolled off)
    if y_offset == 0 {
        Paragraph::new(rounded_top(area.width, border))
            .style(bg_style)
            .render(
                Rect {
                    y: area.y,
                    height: 1,
                    ..area
                },
                buf,
            );
    }

    // Header (skip if scrolled off)
    if y_offset <= 1 {
        Paragraph::new(header).style(bg_style).render(
            Rect {
                y: area.y + 1 - y_offset,
                height: 1,
                ..area
            },
            buf,
        );
    }

    // Bottom border
    if area.height >= 1 {
        let by = area.y + area.height - 1;
        Paragraph::new(rounded_bottom(area.width, border))
            .style(bg_style)
            .render(
                Rect {
                    y: by,
                    height: 1,
                    ..area
                },
                buf,
            );
    }

    BodyArea {
        top: area.y + 2u16.saturating_sub(y_offset),
        bottom: area.y + area.height.saturating_sub(1),
    }
}
