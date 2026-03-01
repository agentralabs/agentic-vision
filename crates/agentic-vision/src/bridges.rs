//! Sister integration bridge traits for AgenticVision.
//!
//! Each bridge defines the interface for integrating with another Agentra sister.
//! Default implementations are no-ops, allowing gradual adoption.
//! Trait-based design ensures Hydra compatibility — swap implementors without refactoring.
//!
//! Note: Vision already has MCP-level bind tools (vision_bind_code, vision_bind_memory,
//! vision_bind_identity, vision_bind_time) in invention_cognition.rs. These core traits
//! provide the underlying integration interface those tools can delegate to.

/// Bridge to agentic-memory for linking captures to memory nodes.
pub trait MemoryBridge: Send + Sync {
    /// Link a visual capture to a memory node
    fn link_to_memory(&self, capture_id: u64, node_id: u64, relationship: &str) -> Result<(), String> {
        let _ = (capture_id, node_id, relationship);
        Err("Memory bridge not connected".to_string())
    }

    /// Store a visual observation as a memory episode
    fn store_observation(&self, description: &str, labels: &[String]) -> Result<u64, String> {
        let _ = (description, labels);
        Err("Memory bridge not connected".to_string())
    }

    /// Query memory for context about a visual element
    fn memory_context(&self, topic: &str, max_results: usize) -> Vec<String> {
        let _ = (topic, max_results);
        Vec::new()
    }
}

/// Bridge to agentic-identity for identity-aware visual captures.
pub trait IdentityBridge: Send + Sync {
    /// Link a capture to an identity receipt
    fn link_to_receipt(&self, capture_id: u64, receipt_id: &str) -> Result<(), String> {
        let _ = (capture_id, receipt_id);
        Err("Identity bridge not connected".to_string())
    }

    /// Verify ownership of a visual capture
    fn verify_capture_owner(&self, capture_id: u64, agent_id: &str) -> bool {
        let _ = (capture_id, agent_id);
        true // Default: trust all
    }

    /// Sign a capture for integrity verification
    fn sign_capture(&self, capture_id: u64, content_hash: &str) -> Result<String, String> {
        let _ = (capture_id, content_hash);
        Err("Identity bridge not connected".to_string())
    }
}

/// Bridge to agentic-time for temporal visual tracking.
pub trait TimeBridge: Send + Sync {
    /// Link a capture to a temporal entity
    fn link_to_temporal(&self, capture_id: u64, entity_id: &str) -> Result<(), String> {
        let _ = (capture_id, entity_id);
        Err("Time bridge not connected".to_string())
    }

    /// Schedule a future capture at a specific time
    fn schedule_capture(&self, description: &str, capture_at: u64) -> Result<String, String> {
        let _ = (description, capture_at);
        Err("Time bridge not connected".to_string())
    }

    /// Get temporal context for a capture (what was happening when it was taken)
    fn temporal_context(&self, timestamp: u64) -> Vec<String> {
        let _ = timestamp;
        Vec::new()
    }
}

/// Bridge to agentic-contract for policy-governed visual captures.
pub trait ContractBridge: Send + Sync {
    /// Check if a capture operation is allowed by policies
    fn check_capture_policy(&self, source: &str, context: &str) -> Result<bool, String> {
        let _ = (source, context);
        Ok(true) // Default: allow all
    }

    /// Record a visual capture for contract audit trail
    fn record_capture(&self, capture_id: u64, description: &str) -> Result<(), String> {
        let _ = (capture_id, description);
        Err("Contract bridge not connected".to_string())
    }
}

/// Bridge to agentic-codebase for code-visual bindings.
pub trait CodebaseBridge: Send + Sync {
    /// Link a capture to a code symbol (rendered_by, styled_by, controlled_by)
    fn link_to_code(&self, capture_id: u64, symbol: &str, binding_type: &str) -> Result<(), String> {
        let _ = (capture_id, symbol, binding_type);
        Err("Codebase bridge not connected".to_string())
    }

    /// Find code symbols responsible for a visual element
    fn find_code_for_visual(&self, selector: &str) -> Vec<String> {
        let _ = selector;
        Vec::new()
    }

    /// Get code context for a visual component
    fn code_context(&self, symbol: &str) -> Option<String> {
        let _ = symbol;
        None
    }
}

/// Bridge to agentic-comm for visual messaging.
pub trait CommBridge: Send + Sync {
    /// Share a capture via comm channel
    fn share_capture(&self, capture_id: u64, channel_id: u64) -> Result<(), String> {
        let _ = (capture_id, channel_id);
        Err("Comm bridge not connected".to_string())
    }

    /// Broadcast a visual regression alert
    fn broadcast_regression_alert(&self, capture_id: u64, details: &str) -> Result<(), String> {
        let _ = (capture_id, details);
        Err("Comm bridge not connected".to_string())
    }
}

/// No-op implementation of all bridges for standalone use.
#[derive(Debug, Clone, Default)]
pub struct NoOpBridges;

impl MemoryBridge for NoOpBridges {}
impl IdentityBridge for NoOpBridges {}
impl TimeBridge for NoOpBridges {}
impl ContractBridge for NoOpBridges {}
impl CodebaseBridge for NoOpBridges {}
impl CommBridge for NoOpBridges {}

/// Configuration for which bridges are active.
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub memory_enabled: bool,
    pub identity_enabled: bool,
    pub time_enabled: bool,
    pub contract_enabled: bool,
    pub codebase_enabled: bool,
    pub comm_enabled: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            memory_enabled: false,
            identity_enabled: false,
            time_enabled: false,
            contract_enabled: false,
            codebase_enabled: false,
            comm_enabled: false,
        }
    }
}

/// Hydra adapter trait — future orchestrator discovery interface.
pub trait HydraAdapter: Send + Sync {
    fn adapter_id(&self) -> &str;
    fn capabilities(&self) -> Vec<String>;
    fn handle_request(&self, method: &str, params: &str) -> Result<String, String>;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_bridges_implements_all_traits() {
        let b = NoOpBridges;
        let _: &dyn MemoryBridge = &b;
        let _: &dyn IdentityBridge = &b;
        let _: &dyn TimeBridge = &b;
        let _: &dyn ContractBridge = &b;
        let _: &dyn CodebaseBridge = &b;
        let _: &dyn CommBridge = &b;
    }

    #[test]
    fn memory_bridge_defaults() {
        let b = NoOpBridges;
        assert!(b.link_to_memory(1, 2, "observed_during").is_err());
        assert!(b.store_observation("desc", &["label".to_string()]).is_err());
        assert!(b.memory_context("ui", 5).is_empty());
    }

    #[test]
    fn identity_bridge_defaults() {
        let b = NoOpBridges;
        assert!(b.link_to_receipt(1, "arec_123").is_err());
        assert!(b.verify_capture_owner(1, "agent-1"));
        assert!(b.sign_capture(1, "hash").is_err());
    }

    #[test]
    fn time_bridge_defaults() {
        let b = NoOpBridges;
        assert!(b.link_to_temporal(1, "dl-1").is_err());
        assert!(b.schedule_capture("screenshot", 1000).is_err());
        assert!(b.temporal_context(1000).is_empty());
    }

    #[test]
    fn contract_bridge_defaults() {
        let b = NoOpBridges;
        assert!(b.check_capture_policy("file", "ctx").unwrap());
        assert!(b.record_capture(1, "desc").is_err());
    }

    #[test]
    fn codebase_bridge_defaults() {
        let b = NoOpBridges;
        assert!(b.link_to_code(1, "Button", "rendered_by").is_err());
        assert!(b.find_code_for_visual(".btn").is_empty());
        assert!(b.code_context("Button").is_none());
    }

    #[test]
    fn comm_bridge_defaults() {
        let b = NoOpBridges;
        assert!(b.share_capture(1, 1).is_err());
        assert!(b.broadcast_regression_alert(1, "details").is_err());
    }

    #[test]
    fn bridge_config_defaults_all_false() {
        let cfg = BridgeConfig::default();
        assert!(!cfg.memory_enabled);
        assert!(!cfg.identity_enabled);
        assert!(!cfg.time_enabled);
        assert!(!cfg.contract_enabled);
        assert!(!cfg.codebase_enabled);
        assert!(!cfg.comm_enabled);
    }

    #[test]
    fn noop_bridges_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NoOpBridges>();
    }

    #[test]
    fn noop_bridges_default_and_clone() {
        let b = NoOpBridges::default();
        let _b2 = b.clone();
    }
}
