//! Main request dispatcher — receives JSON-RPC messages, routes to handlers.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use serde_json::Value;

use crate::prompts::PromptRegistry;
use crate::resources::ResourceRegistry;
use crate::session::VisionSessionManager;
use crate::tools::ToolRegistry;
use crate::types::*;

use super::negotiation::NegotiatedCapabilities;
use super::validator::validate_request;

/// The main protocol handler that dispatches incoming JSON-RPC messages.
pub struct ProtocolHandler {
    session: Arc<Mutex<VisionSessionManager>>,
    capabilities: Arc<Mutex<NegotiatedCapabilities>>,
    shutdown_requested: Arc<AtomicBool>,
    /// Tracks whether an auto-session was started so we can auto-end it.
    auto_session_started: AtomicBool,
}

impl ProtocolHandler {
    pub fn new(session: Arc<Mutex<VisionSessionManager>>) -> Self {
        Self {
            session,
            capabilities: Arc::new(Mutex::new(NegotiatedCapabilities::default())),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            auto_session_started: AtomicBool::new(false),
        }
    }

    /// Returns true once a shutdown request has been handled.
    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::Relaxed)
    }

    pub async fn handle_message(&self, msg: JsonRpcMessage) -> Option<Value> {
        match msg {
            JsonRpcMessage::Request(req) => Some(self.handle_request(req).await),
            JsonRpcMessage::Notification(notif) => {
                self.handle_notification(notif).await;
                None
            }
            _ => {
                tracing::warn!("Received unexpected message type from client");
                None
            }
        }
    }

    /// Cleanup on transport close (EOF). Auto-ends session if one was started.
    pub async fn cleanup(&self) {
        if !self.auto_session_started.load(Ordering::Relaxed) {
            return;
        }

        let mut session = self.session.lock().await;
        match session.end_session() {
            Ok(sid) => {
                tracing::info!("Auto-ended vision session {sid} on EOF");
            }
            Err(e) => {
                tracing::warn!("Failed to auto-end vision session on EOF: {e}");
                if let Err(save_err) = session.save() {
                    tracing::error!("Failed to save vision on EOF cleanup: {save_err}");
                }
            }
        }
        self.auto_session_started.store(false, Ordering::Relaxed);
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> Value {
        if let Err(e) = validate_request(&request) {
            return serde_json::to_value(e.to_json_rpc_error(request.id)).unwrap_or_default();
        }

        let id = request.id.clone();
        let result = self.dispatch_request(&request).await;

        match result {
            Ok(value) => serde_json::to_value(JsonRpcResponse::new(id, value)).unwrap_or_default(),
            Err(e) => serde_json::to_value(e.to_json_rpc_error(id)).unwrap_or_default(),
        }
    }

    async fn dispatch_request(&self, request: &JsonRpcRequest) -> McpResult<Value> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params.clone()).await,
            "shutdown" => self.handle_shutdown().await,

            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(request.params.clone()).await,

            "resources/list" => self.handle_resources_list().await,
            "resources/templates/list" => self.handle_resource_templates_list().await,
            "resources/read" => self.handle_resources_read(request.params.clone()).await,
            "resources/subscribe" => Ok(Value::Object(serde_json::Map::new())),
            "resources/unsubscribe" => Ok(Value::Object(serde_json::Map::new())),

            "prompts/list" => self.handle_prompts_list().await,
            "prompts/get" => self.handle_prompts_get(request.params.clone()).await,

            "ping" => Ok(Value::Object(serde_json::Map::new())),

            _ => Err(McpError::MethodNotFound(request.method.clone())),
        }
    }

    async fn handle_notification(&self, notification: JsonRpcNotification) {
        match notification.method.as_str() {
            "initialized" => {
                let mut caps = self.capabilities.lock().await;
                if let Err(e) = caps.mark_initialized() {
                    tracing::error!("Failed to mark initialized: {e}");
                }

                // Auto-start vision session when client confirms connection.
                let mut session = self.session.lock().await;
                match session.start_session(None) {
                    Ok(sid) => {
                        self.auto_session_started.store(true, Ordering::Relaxed);
                        tracing::info!("Auto-started vision session {sid}");
                    }
                    Err(e) => {
                        tracing::error!("Failed to auto-start vision session: {e}");
                    }
                }
            }
            "notifications/cancelled" | "$/cancelRequest" => {
                tracing::info!("Received cancellation notification");
            }
            _ => {
                tracing::debug!("Unknown notification: {}", notification.method);
            }
        }
    }

    async fn handle_initialize(&self, params: Option<Value>) -> McpResult<Value> {
        let init_params: InitializeParams = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::InvalidParams(e.to_string()))?
            .ok_or_else(|| McpError::InvalidParams("Initialize params required".to_string()))?;

        let mut caps = self.capabilities.lock().await;
        let result = caps.negotiate(init_params)?;

        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    async fn handle_shutdown(&self) -> McpResult<Value> {
        tracing::info!("Shutdown requested");

        let mut session = self.session.lock().await;

        // Auto-end vision session if one was auto-started.
        if self.auto_session_started.swap(false, Ordering::Relaxed) {
            let sid = session.current_session_id();
            match session.end_session() {
                Ok(_) => {
                    tracing::info!("Auto-ended vision session {sid}");
                }
                Err(e) => {
                    tracing::warn!("Failed to auto-end vision session on shutdown: {e}");
                    session.save()?;
                }
            }
        } else {
            session.save()?;
        }

        self.shutdown_requested.store(true, Ordering::Relaxed);
        Ok(Value::Object(serde_json::Map::new()))
    }

    async fn handle_tools_list(&self) -> McpResult<Value> {
        let result = ToolListResult {
            tools: ToolRegistry::list_tools(),
            next_cursor: None,
        };
        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    async fn handle_tools_call(&self, params: Option<Value>) -> McpResult<Value> {
        let call_params: ToolCallParams = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::InvalidParams(e.to_string()))?
            .ok_or_else(|| McpError::InvalidParams("Tool call params required".to_string()))?;

        let tool_name = call_params.name.clone();
        let args_summary = call_params
            .arguments
            .as_ref()
            .map(|a| truncate_json_summary(a, 200))
            .unwrap_or_default();

        // Classify errors: protocol errors (ToolNotFound etc.) become JSON-RPC errors;
        // tool execution errors (CaptureNotFound, VisionError, etc.) become isError: true.
        let result = match ToolRegistry::call(&call_params.name, call_params.arguments, &self.session).await {
            Ok(r) => r,
            Err(e) if e.is_protocol_error() => return Err(e),
            Err(e) => ToolCallResult::error(e.to_string()),
        };

        // Auto-capture tool context into the session log.
        // Skip logging observation_log calls to avoid recursion.
        if tool_name != "observation_log" {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let capture_id = extract_capture_id(&result);
            let record = crate::session::ToolCallRecord {
                tool_name,
                summary: args_summary,
                timestamp: now,
                capture_id,
            };
            let mut session = self.session.lock().await;
            session.log_tool_call(record);
        }

        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    async fn handle_resources_list(&self) -> McpResult<Value> {
        let result = ResourceListResult {
            resources: ResourceRegistry::list_resources(),
            next_cursor: None,
        };
        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    async fn handle_resource_templates_list(&self) -> McpResult<Value> {
        let result = ResourceTemplateListResult {
            resource_templates: ResourceRegistry::list_templates(),
            next_cursor: None,
        };
        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    async fn handle_resources_read(&self, params: Option<Value>) -> McpResult<Value> {
        let read_params: ResourceReadParams = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::InvalidParams(e.to_string()))?
            .ok_or_else(|| McpError::InvalidParams("Resource read params required".to_string()))?;

        let result = ResourceRegistry::read(&read_params.uri, &self.session).await?;

        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    async fn handle_prompts_list(&self) -> McpResult<Value> {
        let result = PromptListResult {
            prompts: PromptRegistry::list_prompts(),
            next_cursor: None,
        };
        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    async fn handle_prompts_get(&self, params: Option<Value>) -> McpResult<Value> {
        let get_params: PromptGetParams = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::InvalidParams(e.to_string()))?
            .ok_or_else(|| McpError::InvalidParams("Prompt get params required".to_string()))?;

        let result = PromptRegistry::get(&get_params.name, get_params.arguments).await?;

        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }
}

/// Truncate a JSON value to a short summary string.
fn truncate_json_summary(value: &Value, max_len: usize) -> String {
    let s = value.to_string();
    if s.len() <= max_len {
        s
    } else {
        format!("{}…", &s[..max_len])
    }
}

/// Try to extract a capture_id from a tool call result.
fn extract_capture_id(result: &crate::types::ToolCallResult) -> Option<u64> {
    for content in &result.content {
        if let crate::types::ToolContent::Text { text } = content {
            if let Ok(v) = serde_json::from_str::<Value>(text) {
                if let Some(id) = v.get("capture_id").and_then(|v| v.as_u64()) {
                    return Some(id);
                }
            }
        }
    }
    None
}
