//! MCP tool implementations.

pub mod observation_log;
pub mod registry;
pub mod session_end;
pub mod session_start;
pub mod vision_capture;
pub mod vision_compare;
pub mod vision_diff;
pub mod vision_health;
pub mod vision_link;
pub mod vision_ocr;
pub mod vision_query;
pub mod vision_similar;
pub mod vision_track;

pub use registry::ToolRegistry;
