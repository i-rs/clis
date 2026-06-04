//! Per-message-type rendering components.
//!
//! Each message variant has its own component implementing [`MessageComponent`].
//! Components own their mutable state (expand/collapse) and handle height
//! calculation and rendering. Click handling is done externally via the
//! scroller which tracks component y-ranges.

mod assistant;
mod error;
mod evaluation;
mod feedback;
mod image;
mod quality;
mod tool_call;
mod user;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::app::Message;
use crate::theme::Theme;

/// Action returned for clickable message elements.
#[derive(Debug, Clone, PartialEq)]
pub enum ClickAction {
    ToggleExpand,
}

/// Unified trait for rendering a single chat message.
pub trait MessageComponent {
    /// Total height (in rows) this component needs at the given width.
    fn height(&self, width: u16) -> u16;

    /// Render the component into `buf` within `area`.
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool);

    /// Whether the first line of this component is clickable (toggle expand).
    fn clickable(&self) -> bool { false }
}

/// Build all components from the app's message list.
pub fn build_components(
    messages: &[Message],
    tool_call_expanded: &std::collections::HashSet<usize>,
    reasoning_expanded: &std::collections::HashSet<usize>,
) -> Vec<Box<dyn MessageComponent>> {
    messages.iter().enumerate().map(|(idx, msg)| build_one(msg, idx, tool_call_expanded, reasoning_expanded)).collect()
}

fn build_one(msg: &Message, idx: usize, tce: &std::collections::HashSet<usize>, re: &std::collections::HashSet<usize>) -> Box<dyn MessageComponent> {
    match msg {
        Message::User { text } => Box::new(user::UserBubble::new(text)),
        Message::Assistant { text, reasoning } => Box::new(assistant::AssistantBlock::new(text, reasoning, re.contains(&idx))),
        Message::ToolCall { name, args, result, step, total_steps } =>
            Box::new(tool_call::ToolCallCard::new(name, args, result, *step, *total_steps, tce.contains(&idx))),
        Message::Error { text } => Box::new(error::ErrorBanner::new(text)),
        Message::Evaluation { tool, valid, issues } => Box::new(evaluation::EvaluationInline::new(tool, *valid, issues)),
        Message::Quality { score, complete, issues, .. } => Box::new(quality::QualityCard::new(*score, *complete, issues)),
        Message::Feedback { positive, message } => Box::new(feedback::FeedbackRow::new(*positive, message.as_deref())),
        Message::Image { path, alt_text, width, height, format } =>
            Box::new(image::ImageCard::new(path, alt_text, *width, *height, format)),
    }
}
