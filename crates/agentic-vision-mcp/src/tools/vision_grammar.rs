//! Grammar tools: vision_grammar_learn, vision_grammar_get, vision_grammar_status,
//! vision_grammar_update, vision_grammar_pin.
//!
//! These tools manage the Site Grammar system — the core invention of the
//! Perception Revolution.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use agentic_vision::perception::grammar::{
    GrammarStatus, IntentRoute, InteractionPattern, NavigationGrammar, SiteGrammar, StateIndicator,
};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

// ── vision_grammar_learn ──

#[derive(Debug, Deserialize)]
struct GrammarLearnParams {
    domain: String,
    content_map: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    interaction_patterns: Vec<InteractionPatternInput>,
    #[serde(default)]
    state_indicators: Vec<StateIndicatorInput>,
    #[serde(default)]
    intent_routes: Vec<IntentRouteInput>,
    navigation_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InteractionPatternInput {
    name: String,
    steps: std::collections::HashMap<String, String>,
    success_indicator: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StateIndicatorInput {
    state_name: String,
    selector: String,
}

#[derive(Debug, Deserialize)]
struct IntentRouteInput {
    intent: String,
    content_keys: Vec<String>,
    interaction: Option<String>,
}

pub fn definition_grammar_learn() -> ToolDefinition {
    ToolDefinition {
        name: "vision_grammar_learn".to_string(),
        description: Some(
            "Learn and store a site grammar for a domain, making future visits free".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "domain": { "type": "string", "description": "Domain to learn grammar for (e.g., 'amazon.com')" },
                "content_map": {
                    "type": "object",
                    "additionalProperties": { "type": "string" },
                    "description": "Map of semantic names to CSS selectors (e.g., {'product_price': '.a-price-whole'})"
                },
                "interaction_patterns": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "steps": { "type": "object", "additionalProperties": { "type": "string" } },
                            "success_indicator": { "type": "string" }
                        },
                        "required": ["name", "steps"]
                    },
                    "description": "Interaction patterns (search, pagination, etc.)"
                },
                "state_indicators": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "state_name": { "type": "string" },
                            "selector": { "type": "string" }
                        },
                        "required": ["state_name", "selector"]
                    }
                },
                "intent_routes": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "intent": { "type": "string" },
                            "content_keys": { "type": "array", "items": { "type": "string" } },
                            "interaction": { "type": "string" }
                        },
                        "required": ["intent", "content_keys"]
                    }
                },
                "navigation_type": {
                    "type": "string",
                    "enum": ["multi_page", "spa", "infinite_scroll", "hybrid"]
                }
            },
            "required": ["domain"]
        }),
    }
}

pub async fn execute_grammar_learn(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: GrammarLearnParams = serde_json::from_value(args)
        .map_err(|e| McpError::InvalidParams(format!("invalid params: {e}")))?;

    let mut session = session.lock().await;
    let domain = params.domain.to_lowercase();

    // Check if grammar already exists
    let existing = session.grammar_store().has(&domain);

    let mut grammar = if existing {
        session
            .grammar_store()
            .get(&domain)
            .cloned()
            .unwrap_or_else(|| SiteGrammar::new(&domain))
    } else {
        SiteGrammar::new(&domain)
    };

    // Apply content map
    if let Some(content_map) = params.content_map {
        for (name, selector) in content_map {
            grammar.add_content(name, selector);
        }
    }

    // Apply interaction patterns
    for ip in params.interaction_patterns {
        grammar.interaction_patterns.push(InteractionPattern {
            name: ip.name,
            steps: ip.steps,
            success_indicator: ip.success_indicator,
        });
    }

    // Apply state indicators
    for si in params.state_indicators {
        grammar.state_indicators.push(StateIndicator {
            state_name: si.state_name,
            selector: si.selector,
        });
    }

    // Apply intent routes
    for ir in params.intent_routes {
        grammar.intent_routes.push(IntentRoute {
            intent: ir.intent,
            content_keys: ir.content_keys,
            interaction: ir.interaction,
        });
    }

    // Apply navigation type
    if let Some(nav_type) = params.navigation_type {
        grammar.navigation = NavigationGrammar {
            navigation_type: match nav_type.as_str() {
                "spa" => agentic_vision::perception::grammar::NavigationType::Spa,
                "infinite_scroll" => {
                    agentic_vision::perception::grammar::NavigationType::InfiniteScroll
                }
                "hybrid" => agentic_vision::perception::grammar::NavigationType::Hybrid,
                _ => agentic_vision::perception::grammar::NavigationType::MultiPage,
            },
            ..grammar.navigation
        };
    }

    let content_count = grammar.content_map.len();
    let route_count = grammar.intent_routes.len();

    session.grammar_store_mut().insert(grammar);
    session.mark_dirty();

    Ok(ToolCallResult::json(&json!({
        "status": if existing { "updated" } else { "learned" },
        "domain": domain,
        "content_map_entries": content_count,
        "intent_routes": route_count,
        "message": format!(
            "Grammar {} for {}. {} content entries, {} intent routes. Future visits will be near-free.",
            if existing { "updated" } else { "learned" },
            domain,
            content_count,
            route_count,
        )
    })))
}

// ── vision_grammar_get ──

pub fn definition_grammar_get() -> ToolDefinition {
    ToolDefinition {
        name: "vision_grammar_get".to_string(),
        description: Some("Get the stored grammar for a known site domain".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "domain": { "type": "string", "description": "Domain to look up (e.g., 'amazon.com')" }
            },
            "required": ["domain"]
        }),
    }
}

pub async fn execute_grammar_get(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("domain required".into()))?
        .to_lowercase();

    let session = session.lock().await;

    match session.grammar_store().get(&domain) {
        Some(grammar) => Ok(ToolCallResult::json(&json!({
            "found": true,
            "grammar": grammar,
        }))),
        None => Ok(ToolCallResult::json(&json!({
            "found": false,
            "domain": domain,
            "available_domains": session.grammar_store().domains(),
        }))),
    }
}

// ── vision_grammar_status ──

pub fn definition_grammar_status() -> ToolDefinition {
    ToolDefinition {
        name: "vision_grammar_status".to_string(),
        description: Some(
            "Check grammar confidence, drift status, and query statistics for a domain".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "domain": { "type": "string", "description": "Domain to check (or omit for all grammars)" }
            }
        }),
    }
}

pub async fn execute_grammar_status(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let domain = args.get("domain").and_then(|v| v.as_str());
    let session = session.lock().await;

    if let Some(domain) = domain {
        let domain = domain.to_lowercase();
        match session.grammar_store().get(&domain) {
            Some(grammar) => Ok(ToolCallResult::json(&json!({
                "domain": grammar.domain,
                "status": grammar.status,
                "grammar_version": grammar.grammar_version,
                "average_confidence": grammar.average_confidence(),
                "success_rate": grammar.success_rate(),
                "query_success_count": grammar.query_success_count,
                "query_failure_count": grammar.query_failure_count,
                "content_map_entries": grammar.content_map.len(),
                "intent_routes": grammar.intent_routes.len(),
                "pinned": grammar.pinned,
                "significance": grammar.significance,
                "last_verified": grammar.last_verified,
            }))),
            None => Ok(ToolCallResult::json(&json!({
                "found": false,
                "domain": domain,
            }))),
        }
    } else {
        // Summary of all grammars
        let store = session.grammar_store();
        let summaries: Vec<_> = store
            .grammars
            .values()
            .map(|g| {
                json!({
                    "domain": g.domain,
                    "status": g.status,
                    "entries": g.content_map.len(),
                    "success_rate": g.success_rate(),
                    "confidence": g.average_confidence(),
                })
            })
            .collect();

        Ok(ToolCallResult::json(&json!({
            "grammar_count": store.count(),
            "active_count": store.active_grammars().len(),
            "drifted_count": store.drifted_grammars().len(),
            "grammars": summaries,
        })))
    }
}

// ── vision_grammar_update ──

pub fn definition_grammar_update() -> ToolDefinition {
    ToolDefinition {
        name: "vision_grammar_update".to_string(),
        description: Some("Force partial or full re-learn of a site grammar".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "domain": { "type": "string", "description": "Domain to update" },
                "content_updates": {
                    "type": "object",
                    "additionalProperties": { "type": "string" },
                    "description": "Map of field names to updated selectors"
                },
                "mark_verified": { "type": "boolean", "description": "Mark grammar as verified against live site" },
                "structural_hash": { "type": "string", "description": "New structural hash from the live page" }
            },
            "required": ["domain"]
        }),
    }
}

pub async fn execute_grammar_update(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("domain required".into()))?
        .to_lowercase();

    let mut session = session.lock().await;

    let grammar = session
        .grammar_store_mut()
        .get_mut(&domain)
        .ok_or_else(|| McpError::InvalidParams(format!("No grammar found for {domain}")))?;

    let mut updates = Vec::new();

    // Apply content updates
    if let Some(content_updates) = args.get("content_updates").and_then(|v| v.as_object()) {
        for (name, selector) in content_updates {
            if let Some(sel) = selector.as_str() {
                grammar.add_content(name.clone(), sel);
                updates.push(format!("updated {name}"));
            }
        }
    }

    // Mark as verified
    if args
        .get("mark_verified")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        grammar.last_verified = Some(now);
        if grammar.status == GrammarStatus::Drifted {
            grammar.status = GrammarStatus::Active;
        }
        updates.push("marked verified".into());
    }

    // Update structural hash
    if let Some(hash) = args.get("structural_hash").and_then(|v| v.as_str()) {
        grammar.structural_hash = Some(hash.to_string());
        updates.push("structural hash updated".into());
    }

    session.mark_dirty();

    Ok(ToolCallResult::json(&json!({
        "domain": domain,
        "updates_applied": updates,
        "status": "updated",
    })))
}

// ── vision_grammar_pin ──

pub fn definition_grammar_pin() -> ToolDefinition {
    ToolDefinition {
        name: "vision_grammar_pin".to_string(),
        description: Some(
            "Pin a grammar version permanently, preventing archival or compression".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "domain": { "type": "string", "description": "Domain to pin" },
                "unpin": { "type": "boolean", "default": false, "description": "Set true to unpin instead" }
            },
            "required": ["domain"]
        }),
    }
}

pub async fn execute_grammar_pin(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("domain required".into()))?
        .to_lowercase();
    let unpin = args.get("unpin").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut session = session.lock().await;
    let grammar = session
        .grammar_store_mut()
        .get_mut(&domain)
        .ok_or_else(|| McpError::InvalidParams(format!("No grammar found for {domain}")))?;

    grammar.pinned = !unpin;
    session.mark_dirty();

    Ok(ToolCallResult::json(&json!({
        "domain": domain,
        "pinned": !unpin,
        "message": if !unpin {
            format!("Grammar for {domain} pinned permanently")
        } else {
            format!("Grammar for {domain} unpinned")
        }
    })))
}
