//! AgenticVision MCP Server — universal LLM access to persistent visual memory.

pub mod config;
pub mod ghost_bridge;
pub mod prompts;
pub mod protocol;
pub mod repl;
pub mod resources;
pub mod session;
pub mod tools;
pub mod transport;
pub mod types;

pub use config::resolve_vision_path;
pub use protocol::ProtocolHandler;
pub use session::VisionSessionManager;
pub use transport::StdioTransport;
