//! Server mode — HTTP API server + Web Dashboard.
//!
//! `claw serve` is the default command. It starts an HTTP server that serves:
//! - REST API endpoints for chat, sessions, agents, stats, etc.
//! - SSE streaming for real-time chat responses
//! - Web Dashboard SPA (unless `--api-only` is set)

pub mod app;
pub mod assets;
pub mod middleware;
pub mod rate_limit;
pub mod routes;

pub use app::AppState;
pub use app::run;
pub use middleware::UserId;
