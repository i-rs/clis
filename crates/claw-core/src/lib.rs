//! i-rs-claw-core: AI agent engine library.
//! Provides AppCore, SessionManager, LLM providers, tools, storage, and memory.

pub mod app;
pub mod config;
pub mod convstore;
pub mod core;
pub mod error;
pub mod llm;
#[cfg(feature = "mcp")]
pub mod mcp;
pub mod memory;
pub mod message;
pub mod plugin;
pub mod providers;
pub mod router;
pub mod semantic;
pub mod session;
pub mod skill_store;
pub mod stats;
pub mod storage;
#[cfg(test)]
pub mod test_helpers;
pub mod theme;
pub mod tool_cache;
pub mod tools;
pub mod utils;
