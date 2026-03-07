//! Perception tools: vision_dom_extract, vision_intent_extract, vision_delta_perceive,
//! vision_scope_capture, vision_perception_route.
//!
//! These tools implement the Adaptive Perception Stack (Layers 0-4).

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

// ── vision_dom_extract (Layer 0) ──

#[derive(Debug, Deserialize)]
struct DomExtractParams {
    url: String,
    #[serde(default)]
    fields: Vec<String>,
    #[serde(default)]
    selectors: Vec<String>,
}

pub fn definition_dom_extract() -> ToolDefinition {
    ToolDefinition {
        name: "vision_dom_extract".to_string(),
        description: Some(
            "Extract structured data from a page via DOM query without screenshot (Layer 0, zero vision tokens)"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "URL of the page to extract from" },
                "fields": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Semantic field names to extract (e.g., 'product_price', 'page_title')"
                },
                "selectors": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "CSS selectors to query directly (alternative to fields)"
                }
            },
            "required": ["url"]
        }),
    }
}

pub async fn execute_dom_extract(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: DomExtractParams = serde_json::from_value(args)
        .map_err(|e| McpError::InvalidParams(format!("invalid params: {e}")))?;

    let session = session.lock().await;

    // Check if we have a grammar for this domain
    let domain = extract_domain(&params.url);
    let grammar = domain
        .as_deref()
        .and_then(|d| session.grammar_store().get(d));

    if let Some(grammar) = grammar {
        // Grammar found — resolve fields to selectors
        let mut results = serde_json::Map::new();
        let mut resolved_selectors = Vec::new();

        for field in &params.fields {
            if let Some(entry) = grammar.content_map.get(field.as_str()) {
                resolved_selectors.push(json!({
                    "field": field,
                    "selector": entry.selector,
                    "selector_type": entry.selector_type,
                    "confidence": entry.confidence,
                    "fallbacks": entry.fallback_selectors,
                }));
            }
        }

        // Also include direct selectors
        for sel in &params.selectors {
            resolved_selectors.push(json!({
                "field": null,
                "selector": sel,
                "selector_type": "css",
                "confidence": null,
            }));
        }

        results.insert("layer".into(), json!("L0_dom_extraction"));
        results.insert("grammar_used".into(), json!(true));
        results.insert("grammar_domain".into(), json!(grammar.domain));
        results.insert("grammar_status".into(), json!(grammar.status));
        results.insert("resolved_selectors".into(), json!(resolved_selectors));
        results.insert("tokens_used".into(), json!(0));
        results.insert(
            "instruction".into(),
            json!(
                "Use the resolved selectors to query the page DOM directly. No screenshot needed."
            ),
        );

        Ok(ToolCallResult::json(&Value::Object(results)))
    } else {
        // No grammar — return guidance for DOM extraction
        Ok(ToolCallResult::json(&json!({
            "layer": "L0_dom_extraction",
            "grammar_used": false,
            "domain": domain,
            "fields_requested": params.fields,
            "selectors_requested": params.selectors,
            "tokens_used": 0,
            "instruction": "No grammar found for this domain. Query the accessibility tree or common selectors. Consider learning a grammar for future visits.",
            "common_selectors": {
                "price": ["[class*=price]", "[itemprop=price]", ".a-price"],
                "title": ["h1", "[itemprop=name]", "#productTitle"],
                "link": ["a[href]", "[role=link]"],
                "button": ["button", "[role=button]", "input[type=submit]"],
                "input": ["input[type=text]", "textarea", "[role=textbox]"]
            }
        })))
    }
}

// ── vision_intent_extract (Layer 2) ──

#[derive(Debug, Deserialize)]
struct IntentExtractParams {
    url: String,
    intent: String,
    #[serde(default)]
    budget_tier: Option<String>,
}

pub fn definition_intent_extract() -> ToolDefinition {
    ToolDefinition {
        name: "vision_intent_extract".to_string(),
        description: Some(
            "Extract data via intent-scoped routing — automatically selects the cheapest perception layer"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "URL of the page" },
                "intent": {
                    "type": "string",
                    "description": "What to extract (e.g., 'find_price', 'check_stock', 'read_content', 'search_products')"
                },
                "budget_tier": {
                    "type": "string",
                    "enum": ["surgical", "focused", "contextual", "visual"],
                    "description": "Token budget tier (defaults to auto-selection based on intent)"
                }
            },
            "required": ["url", "intent"]
        }),
    }
}

pub async fn execute_intent_extract(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: IntentExtractParams = serde_json::from_value(args)
        .map_err(|e| McpError::InvalidParams(format!("invalid params: {e}")))?;

    let session = session.lock().await;
    let domain = extract_domain(&params.url);

    // Check grammar for intent route
    let grammar = domain
        .as_deref()
        .and_then(|d| session.grammar_store().get(d));

    if let Some(grammar) = grammar {
        if let Some(route) = grammar.route_intent(&params.intent) {
            let selectors: Vec<_> = route
                .content_keys
                .iter()
                .filter_map(|key| {
                    grammar.content_map.get(key.as_str()).map(|entry| {
                        json!({
                            "key": key,
                            "selector": entry.selector,
                            "confidence": entry.confidence,
                            "fallbacks": entry.fallback_selectors,
                        })
                    })
                })
                .collect();

            return Ok(ToolCallResult::json(&json!({
                "layer": "L1_grammar_lookup",
                "intent": params.intent,
                "grammar_domain": grammar.domain,
                "grammar_version": grammar.grammar_version,
                "route_found": true,
                "selectors": selectors,
                "interaction": route.interaction,
                "estimated_tokens": 0,
                "instruction": "Use the resolved selectors from the grammar. No screenshot needed."
            })));
        }
    }

    // No grammar or no route — provide intent-scoped guidance
    let (layer, estimated_tokens, instruction) = match params.intent.as_str() {
        "find_price" | "check_stock" | "check_status" => (
            "L0_dom_extraction",
            15u32,
            "Query DOM with common price/stock selectors",
        ),
        "search_products" | "submit_form" | "navigate" => (
            "L1_grammar_lookup",
            25,
            "Extract interaction elements from accessibility tree",
        ),
        "read_content" | "read_article" | "list_items" => (
            "L2_intent_scoped",
            200,
            "Extract main content area text only",
        ),
        "monitor_changes" | "track_price" => (
            "L3_delta_vision",
            50,
            "Compare current DOM against stored baseline",
        ),
        "analyze_chart" | "analyze_image" | "verify_visual" => (
            "L4_scoped_screenshot",
            400,
            "Take a scoped screenshot of the relevant element only",
        ),
        _ => (
            "L0_dom_extraction",
            15,
            "Unknown intent — defaulting to DOM extraction",
        ),
    };

    Ok(ToolCallResult::json(&json!({
        "layer": layer,
        "intent": params.intent,
        "grammar_domain": domain,
        "route_found": false,
        "estimated_tokens": estimated_tokens,
        "instruction": instruction,
        "suggestion": "Consider using vision_grammar_learn to create a grammar for this site"
    })))
}

// ── vision_perception_route (meta-tool: shows routing decision) ──

pub fn definition_perception_route() -> ToolDefinition {
    ToolDefinition {
        name: "vision_perception_route".to_string(),
        description: Some(
            "Show which perception layer would handle a request and estimated token cost"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "URL of the page" },
                "intent": { "type": "string", "description": "What to perceive" }
            },
            "required": ["url", "intent"]
        }),
    }
}

pub async fn execute_perception_route(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let url = args
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("url required".into()))?;
    let intent = args
        .get("intent")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("intent required".into()))?;

    let session = session.lock().await;
    let domain = extract_domain(url);
    let has_grammar = domain
        .as_deref()
        .is_some_and(|d| session.grammar_store().has(d));

    let grammar_info = if has_grammar {
        let g = session
            .grammar_store()
            .get(domain.as_deref().unwrap_or_default())
            .ok_or_else(|| McpError::InternalError("grammar not found for domain".into()))?;
        json!({
            "domain": g.domain,
            "status": g.status,
            "content_map_entries": g.content_map.len(),
            "intent_routes": g.intent_routes.len(),
            "success_rate": g.success_rate(),
            "has_route_for_intent": g.route_intent(intent).is_some(),
        })
    } else {
        json!(null)
    };

    // Get cache stats
    let cache_stats = session.intent_cache().stats();

    Ok(ToolCallResult::json(&json!({
        "url": url,
        "domain": domain,
        "intent": intent,
        "has_grammar": has_grammar,
        "grammar": grammar_info,
        "cache_stats": {
            "entries": cache_stats.entry_count,
            "hit_rate": cache_stats.hit_rate,
            "total_tokens_saved": cache_stats.total_tokens_saved,
        },
        "routing": {
            "would_use_screenshot": !has_grammar && matches!(intent, "analyze_chart" | "analyze_image"),
            "estimated_tokens": if has_grammar { 0 } else { 15 },
            "recommendation": if has_grammar {
                "Grammar found — use vision_intent_extract for zero-token extraction"
            } else {
                "No grammar — use vision_dom_extract, then vision_grammar_learn to save future visits"
            }
        }
    })))
}

// ── Helpers ──

fn extract_domain(url: &str) -> Option<String> {
    let url = url.trim();
    let after_proto = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };
    let domain = if let Some(pos) = after_proto.find('/') {
        &after_proto[..pos]
    } else {
        after_proto
    };
    let domain = if let Some(pos) = domain.find(':') {
        &domain[..pos]
    } else {
        domain
    };
    if domain.is_empty() {
        None
    } else {
        Some(domain.to_lowercase())
    }
}
