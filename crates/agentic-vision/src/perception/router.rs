//! Adaptive Perception Router — selects the cheapest layer for each request.
//!
//! The Escalation Contract:
//! 1. NEVER take a screenshot when a DOM query can answer the question.
//! 2. NEVER run a full DOM scan when a grammar query can answer it.
//! 3. NEVER re-extract what was just extracted and cached.
//! 4. NEVER pay full-page cost when scoped extraction is sufficient.
//! 5. ALWAYS declare intent BEFORE choosing perception method.

use serde::{Deserialize, Serialize};

use super::budget::TokenBudgetTier;
use super::cache::{IntentCache, IntentCacheKey};
use super::grammar::GrammarStore;
use super::types::{PerceptionIntent, PerceptionRequest};

/// Which perception layer handled the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerceptionLayer {
    /// Layer 0: Semantic DOM Extraction (0 vision tokens).
    DomExtraction,
    /// Layer 1: Site Grammar lookup (amortized to near-zero).
    GrammarLookup,
    /// Layer 2: Intent-Scoped Extraction (proportional cost).
    IntentScoped,
    /// Layer 3: Delta Vision (only changed content).
    DeltaVision,
    /// Layer 4: Scoped Screenshot (visual content only).
    ScopedScreenshot,
}

impl PerceptionLayer {
    /// Numeric layer index (0-4).
    pub fn index(self) -> u8 {
        match self {
            Self::DomExtraction => 0,
            Self::GrammarLookup => 1,
            Self::IntentScoped => 2,
            Self::DeltaVision => 3,
            Self::ScopedScreenshot => 4,
        }
    }

    /// Typical token cost for this layer.
    pub fn typical_tokens(self) -> u32 {
        match self {
            Self::DomExtraction => 15,
            Self::GrammarLookup => 0,
            Self::IntentScoped => 100,
            Self::DeltaVision => 50,
            Self::ScopedScreenshot => 400,
        }
    }

    /// The budget tier appropriate for this layer.
    pub fn budget_tier(self) -> TokenBudgetTier {
        match self {
            Self::DomExtraction => TokenBudgetTier::Surgical,
            Self::GrammarLookup => TokenBudgetTier::Surgical,
            Self::IntentScoped => TokenBudgetTier::Focused,
            Self::DeltaVision => TokenBudgetTier::Focused,
            Self::ScopedScreenshot => TokenBudgetTier::Visual,
        }
    }
}

/// Result of the perception router's decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionResult {
    /// Which layer handled the request.
    pub layer: PerceptionLayer,

    /// Extracted data (key-value pairs).
    pub data: serde_json::Value,

    /// Whether the result came from cache.
    pub from_cache: bool,

    /// Tokens actually used.
    pub tokens_used: u32,

    /// Tokens that would have been used with screenshot approach.
    pub tokens_saved: u32,

    /// Whether a grammar was found and used.
    pub grammar_used: bool,

    /// Domain of the site (if identified).
    pub domain: Option<String>,
}

/// The Adaptive Perception Router.
pub struct PerceptionRouter;

impl PerceptionRouter {
    /// Determine the optimal perception layer for a request.
    ///
    /// This is the routing decision logic — it does NOT execute perception.
    /// Execution happens in the MCP tool layer with actual browser access.
    pub fn route(
        request: &PerceptionRequest,
        grammar_store: &GrammarStore,
        _intent_cache: &mut IntentCache,
    ) -> RoutingDecision {
        let domain = request.domain.as_deref();

        // Step 1: Check intent cache first (cheapest possible path)
        if let Some(ref url) = request.url {
            let intent_type = intent_type_string(&request.intent);
            // Try to find a cache hit with any known hashes
            // (actual hash comparison happens at lookup time)
            let _cache_key = IntentCacheKey::new(url, &intent_type, "", "");
            // Real implementation would check with actual hashes from live page.
            // Cache lookup requires structural_hash and content_hash from the live
            // page, which are only available after browser connection.
        }

        // Step 2: Check if we have a grammar for this domain
        let has_grammar = domain.is_some_and(|d| grammar_store.has(d));

        // Step 3: Route based on intent type
        match &request.intent {
            PerceptionIntent::ExtractData { fields } => {
                if has_grammar {
                    // Grammar exists — use grammar lookup (Layer 1) → DOM query (Layer 0)
                    RoutingDecision {
                        primary_layer: PerceptionLayer::GrammarLookup,
                        fallback_layer: Some(PerceptionLayer::DomExtraction),
                        needs_browser: true,
                        needs_screenshot: false,
                        estimated_tokens: fields.len() as u32 * 5,
                        reason: "Grammar found — using grammar-guided DOM extraction".into(),
                    }
                } else {
                    // No grammar — use DOM extraction (Layer 0), learn grammar as side effect
                    RoutingDecision {
                        primary_layer: PerceptionLayer::DomExtraction,
                        fallback_layer: Some(PerceptionLayer::IntentScoped),
                        needs_browser: true,
                        needs_screenshot: false,
                        estimated_tokens: 15 + fields.len() as u32 * 5,
                        reason: "No grammar — using DOM extraction, will learn grammar".into(),
                    }
                }
            }

            PerceptionIntent::FindInteractable { .. } => {
                if has_grammar {
                    RoutingDecision {
                        primary_layer: PerceptionLayer::GrammarLookup,
                        fallback_layer: Some(PerceptionLayer::DomExtraction),
                        needs_browser: true,
                        needs_screenshot: false,
                        estimated_tokens: 10,
                        reason: "Grammar found — using interaction pattern lookup".into(),
                    }
                } else {
                    RoutingDecision {
                        primary_layer: PerceptionLayer::DomExtraction,
                        fallback_layer: Some(PerceptionLayer::ScopedScreenshot),
                        needs_browser: true,
                        needs_screenshot: false,
                        estimated_tokens: 25,
                        reason: "No grammar — scanning DOM for interactive elements".into(),
                    }
                }
            }

            PerceptionIntent::VerifyState { .. } => RoutingDecision {
                primary_layer: if has_grammar {
                    PerceptionLayer::GrammarLookup
                } else {
                    PerceptionLayer::DomExtraction
                },
                fallback_layer: Some(PerceptionLayer::ScopedScreenshot),
                needs_browser: true,
                needs_screenshot: false,
                estimated_tokens: 10,
                reason: "State verification via grammar/DOM indicators".into(),
            },

            PerceptionIntent::ReadContent { .. } => RoutingDecision {
                primary_layer: PerceptionLayer::IntentScoped,
                fallback_layer: Some(PerceptionLayer::ScopedScreenshot),
                needs_browser: true,
                needs_screenshot: false,
                estimated_tokens: 200,
                reason: "Content reading via intent-scoped extraction".into(),
            },

            PerceptionIntent::MonitorChanges { .. } => RoutingDecision {
                primary_layer: PerceptionLayer::DeltaVision,
                fallback_layer: Some(PerceptionLayer::IntentScoped),
                needs_browser: true,
                needs_screenshot: false,
                estimated_tokens: 50,
                reason: "Change monitoring via delta vision".into(),
            },

            PerceptionIntent::AnalyzeVisual
            | PerceptionIntent::ReadDocument
            | PerceptionIntent::VerifyCaptcha => RoutingDecision {
                primary_layer: PerceptionLayer::ScopedScreenshot,
                fallback_layer: None,
                needs_browser: true,
                needs_screenshot: true,
                estimated_tokens: 400,
                reason: "Visual content requires scoped screenshot".into(),
            },

            PerceptionIntent::CaptureFullVisual => RoutingDecision {
                primary_layer: PerceptionLayer::ScopedScreenshot,
                fallback_layer: None,
                needs_browser: true,
                needs_screenshot: true,
                estimated_tokens: 2000,
                reason: "Full visual capture explicitly requested".into(),
            },
        }
    }
}

/// The router's decision about how to handle a perception request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Primary layer to use.
    pub primary_layer: PerceptionLayer,
    /// Fallback layer if primary fails.
    pub fallback_layer: Option<PerceptionLayer>,
    /// Whether a browser connection is needed.
    pub needs_browser: bool,
    /// Whether a screenshot is needed.
    pub needs_screenshot: bool,
    /// Estimated token cost.
    pub estimated_tokens: u32,
    /// Human-readable reason for this routing.
    pub reason: String,
}

/// Convert a PerceptionIntent to a string identifier for cache keys.
fn intent_type_string(intent: &PerceptionIntent) -> String {
    match intent {
        PerceptionIntent::ExtractData { .. } => "extract_data".into(),
        PerceptionIntent::FindInteractable { .. } => "find_interactable".into(),
        PerceptionIntent::VerifyState { .. } => "verify_state".into(),
        PerceptionIntent::ReadContent { .. } => "read_content".into(),
        PerceptionIntent::MonitorChanges { .. } => "monitor_changes".into(),
        PerceptionIntent::AnalyzeVisual => "analyze_visual".into(),
        PerceptionIntent::ReadDocument => "read_document".into(),
        PerceptionIntent::VerifyCaptcha => "verify_captcha".into(),
        PerceptionIntent::CaptureFullVisual => "capture_full_visual".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perception::grammar::SiteGrammar;
    use crate::perception::types::DataField;

    #[test]
    fn test_route_extract_with_grammar() {
        let mut store = GrammarStore::new();
        let mut grammar = SiteGrammar::new("amazon.com");
        grammar.add_content("price", ".a-price-whole");
        store.insert(grammar);

        let mut cache = IntentCache::new();
        let request = PerceptionRequest::extract_data(
            "https://amazon.com/dp/X",
            vec![DataField {
                name: "price".into(),
                field_type: Default::default(),
            }],
        );

        let decision = PerceptionRouter::route(&request, &store, &mut cache);
        assert_eq!(decision.primary_layer, PerceptionLayer::GrammarLookup);
        assert!(!decision.needs_screenshot);
    }

    #[test]
    fn test_route_extract_without_grammar() {
        let store = GrammarStore::new();
        let mut cache = IntentCache::new();
        let request = PerceptionRequest::extract_data(
            "https://unknown-site.com",
            vec![DataField {
                name: "title".into(),
                field_type: Default::default(),
            }],
        );

        let decision = PerceptionRouter::route(&request, &store, &mut cache);
        assert_eq!(decision.primary_layer, PerceptionLayer::DomExtraction);
    }

    #[test]
    fn test_route_visual_always_screenshot() {
        let store = GrammarStore::new();
        let mut cache = IntentCache::new();
        let request = PerceptionRequest {
            intent: PerceptionIntent::AnalyzeVisual,
            url: Some("https://chart-site.com".into()),
            domain: Some("chart-site.com".into()),
            budget: Default::default(),
            fallback: Default::default(),
        };

        let decision = PerceptionRouter::route(&request, &store, &mut cache);
        assert_eq!(decision.primary_layer, PerceptionLayer::ScopedScreenshot);
        assert!(decision.needs_screenshot);
    }

    #[test]
    fn test_route_monitoring() {
        let store = GrammarStore::new();
        let mut cache = IntentCache::new();
        let request = PerceptionRequest::monitor("https://example.com", None);

        let decision = PerceptionRouter::route(&request, &store, &mut cache);
        assert_eq!(decision.primary_layer, PerceptionLayer::DeltaVision);
    }

    #[test]
    fn test_layer_properties() {
        assert_eq!(PerceptionLayer::DomExtraction.index(), 0);
        assert_eq!(PerceptionLayer::ScopedScreenshot.index(), 4);
        assert_eq!(PerceptionLayer::GrammarLookup.typical_tokens(), 0);
    }
}
