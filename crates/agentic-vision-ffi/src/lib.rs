//! Minimal FFI facade for AgenticVision.

/// Crate version exposed for foreign runtimes.
pub fn agentic_vision_ffi_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
