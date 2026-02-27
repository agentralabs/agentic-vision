//! Error types and JSON-RPC error codes for the MCP server.

use super::message::{JsonRpcError, JsonRpcErrorObject, RequestId, JSONRPC_VERSION};

/// Standard JSON-RPC 2.0 error codes.
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// MCP-specific error codes.
pub mod mcp_error_codes {
    pub const REQUEST_CANCELLED: i32 = -32800;
    pub const CONTENT_TOO_LARGE: i32 = -32801;
    pub const RESOURCE_NOT_FOUND: i32 = -32802;
    pub const TOOL_NOT_FOUND: i32 = -32803;
    pub const PROMPT_NOT_FOUND: i32 = -32804;
    pub const CAPTURE_NOT_FOUND: i32 = -32850;
    pub const SESSION_NOT_FOUND: i32 = -32851;
    pub const VISION_ERROR: i32 = -32852;

    /// Server: Unauthorized (missing or invalid bearer token).
    pub const UNAUTHORIZED: i32 = -32900;
    /// Server: User not found (multi-tenant, missing X-User-ID header).
    pub const USER_NOT_FOUND: i32 = -32901;
    /// Server: Rate limited.
    pub const RATE_LIMITED: i32 = -32902;
}

/// All errors that can occur in the MCP server.
#[derive(thiserror::Error, Debug)]
pub enum McpError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invalid params: {0}")]
    InvalidParams(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Request cancelled")]
    RequestCancelled,

    #[error("Content too large: {size} bytes exceeds {max} bytes")]
    ContentTooLarge { size: usize, max: usize },

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Prompt not found: {0}")]
    PromptNotFound(String),

    #[error("Capture not found: {0}")]
    CaptureNotFound(u64),

    #[error("Session not found: {0}")]
    SessionNotFound(u32),

    #[error("Vision error: {0}")]
    VisionError(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Unauthorized — missing or invalid bearer token.
    #[error("Unauthorized")]
    Unauthorized,

    /// User not found — missing X-User-ID header in multi-tenant mode.
    #[error("User not found: {0}")]
    UserNotFound(String),
}

impl McpError {
    /// Returns true if this is a protocol-level error (should be a JSON-RPC error).
    /// Tool execution errors (capture not found, vision error, etc.) return false
    /// and should use `ToolCallResult::error()` with `isError: true` instead.
    pub fn is_protocol_error(&self) -> bool {
        matches!(
            self,
            McpError::ParseError(_)
                | McpError::InvalidRequest(_)
                | McpError::MethodNotFound(_)
                | McpError::ToolNotFound(_)
                | McpError::RequestCancelled
                | McpError::ContentTooLarge { .. }
                | McpError::ResourceNotFound(_)
                | McpError::PromptNotFound(_)
                | McpError::Unauthorized
                | McpError::UserNotFound(_)
        )
    }

    pub fn code(&self) -> i32 {
        use error_codes::*;
        use mcp_error_codes::*;
        match self {
            McpError::ParseError(_) => PARSE_ERROR,
            McpError::InvalidRequest(_) => INVALID_REQUEST,
            McpError::MethodNotFound(_) => METHOD_NOT_FOUND,
            McpError::InvalidParams(_) => INVALID_PARAMS,
            McpError::InternalError(_) => INTERNAL_ERROR,
            McpError::RequestCancelled => REQUEST_CANCELLED,
            McpError::ContentTooLarge { .. } => CONTENT_TOO_LARGE,
            McpError::ResourceNotFound(_) => RESOURCE_NOT_FOUND,
            McpError::ToolNotFound(_) => TOOL_NOT_FOUND,
            McpError::PromptNotFound(_) => PROMPT_NOT_FOUND,
            McpError::CaptureNotFound(_) => CAPTURE_NOT_FOUND,
            McpError::SessionNotFound(_) => SESSION_NOT_FOUND,
            McpError::VisionError(_) => VISION_ERROR,
            McpError::Transport(_) | McpError::Io(_) => INTERNAL_ERROR,
            McpError::Json(_) => PARSE_ERROR,
            McpError::Unauthorized => UNAUTHORIZED,
            McpError::UserNotFound(_) => USER_NOT_FOUND,
        }
    }

    pub fn to_json_rpc_error(&self, id: RequestId) -> JsonRpcError {
        JsonRpcError {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            error: JsonRpcErrorObject {
                code: self.code(),
                message: self.to_string(),
                data: None,
            },
        }
    }
}

impl From<agentic_vision::VisionError> for McpError {
    fn from(e: agentic_vision::VisionError) -> Self {
        McpError::VisionError(e.to_string())
    }
}

pub type McpResult<T> = Result<T, McpError>;
