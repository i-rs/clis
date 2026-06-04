mod key;
mod llm;
mod mouse;
mod overlay;

pub use key::KeyEventHandler;
pub use llm::LlmEventHandler;
pub use mouse::MouseEventHandler;

pub enum Action {
    Continue,
    Quit,
}

type AppMessage = crate::app::Message;
