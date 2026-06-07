//! Per-message-type rendering components.

mod assistant;
mod error;
mod evaluation;
mod feedback;
mod image;
mod quality;
mod style;
mod tool_call;
mod user;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::app::Message;
use i_rs_claw_core::theme::Theme;

/// Operations the App can apply to a component after construction.
/// The default impl for `apply` is a no-op, so each component only
/// has to opt in to the variants it cares about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentOp {
    /// Append streamed text delta to the assistant body.
    AppendText(String),
    /// Append streamed reasoning delta.
    AppendReasoning(String),
    /// Flip the component's expand/collapse state.
    Toggle,
    /// Flip just the Args sub-section inside an expanded ToolCallCard.
    ToggleArgs,
    /// Flip just the Result sub-section inside an expanded ToolCallCard.
    ToggleResult,
}

pub trait MessageComponent {
    fn height(&self, width: u16) -> u16;
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool);
    /// Whether the whole block should be a click target (e.g. a tool
    /// call or an assistant message with reasoning). Components opt
    /// in by returning `true` here.
    fn clickable(&self) -> bool {
        false
    }
    /// Apply a mutation coming from outside (streaming tokens, click
    /// events, etc.). Default impl is a no-op.
    fn apply(&mut self, _op: ComponentOp) {}

    /// Sub-regions within the component that act as click targets.
    /// Each tuple is (y_offset, height, op). When non-empty the
    /// Scroller registers these regions *instead* of the full-area
    /// click so that individual sub-sections (e.g. Args / Result
    /// headers) can receive their own ComponentOp.
    fn extra_click_targets(&self, _width: u16) -> Vec<(u16, u16, ComponentOp)> {
        vec![]
    }
}

pub fn build_component_for(msg: &Message) -> Box<dyn MessageComponent> {
    match msg {
        Message::User { text } if !text.is_empty() => Box::new(user::UserBubble::new(text, None)),
        Message::User { .. } => Box::new(EmptyComponent),
        Message::Assistant {
            text,
            reasoning,
            token_usage,
            ..
        } if !text.is_empty() || !reasoning.is_empty() => Box::new(assistant::AssistantBlock::new(
            text,
            reasoning,
            false,
            None,
            *token_usage,
        )),
        Message::Assistant { .. } => Box::new(EmptyComponent),
        Message::ToolCall {
            name,
            args,
            result,
            step,
            total_steps,
        } => Box::new(tool_call::ToolCallCard::new(
            name,
            args,
            result,
            *step,
            *total_steps,
            false,
            None,
        )),
        Message::Error { text } => Box::new(error::ErrorBanner::new(text, None)),
        Message::Evaluation {
            tool,
            valid,
            issues,
        } if !*valid => Box::new(evaluation::EvaluationInline::new(
            tool, *valid, issues, None,
        )),
        Message::Evaluation { .. } => Box::new(EmptyComponent),
        Message::Quality {
            score,
            complete,
            issues,
            ..
        } => Box::new(quality::QualityCard::new(*score, *complete, issues, None)),
        Message::Feedback { positive, message } => Box::new(feedback::FeedbackRow::new(
            *positive,
            message.as_deref(),
            None,
        )),
        Message::Image {
            path: _path,
            alt_text,
            width,
            height,
            format: _format,
        } => Box::new(image::ImageCard::new(
            alt_text,
            *width as u16,
            *height as u16,
            None,
        )),
    }
}

struct EmptyComponent;
impl MessageComponent for EmptyComponent {
    fn height(&self, _w: u16) -> u16 {
        0
    }
    fn render(&self, _area: Rect, _buf: &mut Buffer, _theme: &Theme, _selected: bool) {}
}
