//! Per-message-type rendering components.

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

#[derive(Debug, Clone, PartialEq)]
pub enum ClickAction { ToggleExpand }

pub trait MessageComponent {
    fn height(&self, width: u16) -> u16;
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool);
    fn clickable(&self) -> bool { false }
}

pub fn build_components(
    messages: &[Message],
    tool_call_expanded: &std::collections::HashSet<usize>,
    reasoning_expanded: &std::collections::HashSet<usize>,
) -> Vec<Box<dyn MessageComponent>> {
    messages.iter().enumerate().map(|(idx, msg)| build_one(msg, idx, tool_call_expanded, reasoning_expanded)).collect()
}

struct EmptyComponent;
impl MessageComponent for EmptyComponent {
    fn height(&self, _w: u16) -> u16 { 0 }
    fn render(&self, _area: Rect, _buf: &mut Buffer, _theme: &Theme, _selected: bool) {}
}

fn build_one(msg: &Message, idx: usize, tce: &std::collections::HashSet<usize>, re: &std::collections::HashSet<usize>) -> Box<dyn MessageComponent> {
    match msg {
        Message::User { text } if !text.is_empty() => Box::new(user::UserBubble::new(text)),
        Message::User { .. } => Box::new(EmptyComponent),
        Message::Assistant { text, reasoning } if !text.is_empty() || !reasoning.is_empty() =>
            Box::new(assistant::AssistantBlock::new(text, reasoning, re.contains(&idx))),
        Message::Assistant { .. } => Box::new(EmptyComponent),
        Message::ToolCall { name, args, result, step, total_steps } =>
            Box::new(tool_call::ToolCallCard::new(name, args, result, *step, *total_steps, tce.contains(&idx))),
        Message::Error { text } => Box::new(error::ErrorBanner::new(text)),
        Message::Evaluation { tool, valid, issues } if !*valid =>
            Box::new(evaluation::EvaluationInline::new(tool, *valid, issues)),
        Message::Evaluation { .. } => Box::new(EmptyComponent),
        Message::Quality { score, complete, issues, .. } =>
            Box::new(quality::QualityCard::new(*score, *complete, issues)),
        Message::Feedback { positive, message } =>
            Box::new(feedback::FeedbackRow::new(*positive, message.as_deref())),
        Message::Image { path, alt_text, width, height, format } =>
            Box::new(image::ImageCard::new(path, alt_text, *width, *height, format)),
    }
}
