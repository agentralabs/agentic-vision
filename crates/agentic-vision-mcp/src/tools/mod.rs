//! MCP tool implementations.

pub mod observation_log;
pub mod registry;
pub mod session_end;
pub mod session_start;
pub mod vision_capture;
pub mod vision_compare;
pub mod vision_diff;
pub mod vision_evidence;
pub mod vision_ground;
pub mod vision_health;
pub mod vision_link;
pub mod vision_ocr;
pub mod vision_query;
pub mod vision_similar;
pub mod vision_suggest;
pub mod vision_track;
pub mod vision_workspace_add;
pub mod vision_workspace_compare;
pub mod vision_workspace_create;
pub mod vision_workspace_list;
pub mod vision_workspace_query;
pub mod vision_workspace_xref;

// V3: 16 Perception Inventions
pub mod invention_cognition;
pub mod invention_grounding;
pub mod invention_prediction;
pub mod invention_temporal;

pub use registry::ToolRegistry;
