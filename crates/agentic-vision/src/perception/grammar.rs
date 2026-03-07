//! Site Grammar System — the core invention of the Perception Revolution.
//!
//! A Site Grammar is the complete behavioral fingerprint of a website: selectors,
//! interaction patterns, state indicators, navigation model, and intent routes.
//! Once learned, a grammar makes all future visits to that site nearly free.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The type of CSS/DOM selector used in a grammar field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SelectorType {
    Css,
    Xpath,
    Aria,
    Text,
    ShadowDom,
}

impl Default for SelectorType {
    fn default() -> Self {
        Self::Css
    }
}

/// A single selector entry with confidence and fallbacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMapEntry {
    /// Primary selector string.
    pub selector: String,

    /// Type of selector.
    #[serde(default)]
    pub selector_type: SelectorType,

    /// Grammar format version that produced this selector.
    #[serde(default = "default_grammar_format")]
    pub format_version: String,

    /// Confidence score [0.0, 1.0] based on successful extractions.
    #[serde(default = "default_confidence")]
    pub confidence: f32,

    /// When this selector was last verified against a live page.
    pub verified_at: Option<u64>,

    /// Fallback selectors if primary fails.
    #[serde(default)]
    pub fallback_selectors: Vec<String>,
}

fn default_grammar_format() -> String {
    "sgr-v1".to_string()
}

fn default_confidence() -> f32 {
    0.5
}

impl ContentMapEntry {
    /// Create a new entry with default confidence.
    pub fn new(selector: impl Into<String>) -> Self {
        Self {
            selector: selector.into(),
            selector_type: SelectorType::Css,
            format_version: "sgr-v1".to_string(),
            confidence: 0.5,
            verified_at: None,
            fallback_selectors: Vec::new(),
        }
    }

    /// Record a successful extraction (boost confidence).
    pub fn record_success(&mut self) {
        self.confidence = (self.confidence + 0.1).min(1.0);
        self.verified_at = Some(now_secs());
    }

    /// Record a failed extraction (reduce confidence).
    pub fn record_failure(&mut self) {
        self.confidence = (self.confidence - 0.2).max(0.0);
    }

    /// Whether this entry is considered reliable.
    pub fn is_reliable(&self) -> bool {
        self.confidence >= 0.8
    }
}

/// An interaction pattern for a specific site action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionPattern {
    /// Descriptive name (e.g., "search", "add_to_cart", "pagination").
    pub name: String,

    /// Map of step names to selectors (e.g., "input" -> "#search-box").
    pub steps: HashMap<String, String>,

    /// Success indicator selector (optional).
    pub success_indicator: Option<String>,
}

/// A state indicator for detecting page state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateIndicator {
    /// What state this detects (e.g., "loading", "error", "logged_in").
    pub state_name: String,

    /// Selector to check for this state.
    pub selector: String,
}

/// How the site navigates between pages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavigationType {
    #[default]
    MultiPage,
    Spa,
    InfiniteScroll,
    Hybrid,
}

/// Navigation grammar — how to move around the site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationGrammar {
    #[serde(default)]
    pub navigation_type: NavigationType,
    #[serde(default)]
    pub js_required: bool,
    #[serde(default)]
    pub spa_navigation: bool,
    #[serde(default = "default_true")]
    pub back_button_safe: bool,
    pub session_state: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for NavigationGrammar {
    fn default() -> Self {
        Self {
            navigation_type: NavigationType::MultiPage,
            js_required: false,
            spa_navigation: false,
            back_button_safe: true,
            session_state: None,
        }
    }
}

/// An intent route: maps a user intent to grammar content_map entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentRoute {
    /// The intent name (e.g., "find_price", "add_to_cart").
    pub intent: String,

    /// Content map keys to query for this intent.
    pub content_keys: Vec<String>,

    /// Interaction pattern name to use (if action-oriented).
    pub interaction: Option<String>,
}

/// Lifecycle status of a grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrammarStatus {
    /// Being learned from first visit.
    Learning,
    /// Actively used with high confidence.
    Active,
    /// Site has changed; partial re-learning needed.
    Drifted,
    /// Not visited in 1+ years; moved to cold storage.
    Archived,
    /// Superseded by newer version; kept for history.
    Historical,
}

/// A complete Site Grammar Record — the behavioral fingerprint of a website.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteGrammar {
    /// Domain this grammar applies to (e.g., "amazon.com").
    pub domain: String,

    /// Grammar version identifier (e.g., "2026-Q1-v2").
    pub grammar_version: String,

    /// When this grammar was last verified against the live site.
    pub last_verified: Option<u64>,

    /// Structural hash of the page (blake3) for drift detection.
    pub structural_hash: Option<String>,

    /// Current lifecycle status.
    #[serde(default = "default_status")]
    pub status: GrammarStatus,

    /// When this grammar was first created.
    pub created_at: u64,

    /// When this grammar was last updated.
    pub updated_at: u64,

    // ── Content Map: what lives where ──
    /// Named selectors for content extraction (e.g., "product_price" -> selector).
    #[serde(default)]
    pub content_map: HashMap<String, ContentMapEntry>,

    // ── Behavioral Grammar: how to interact ──
    /// Named interaction patterns (e.g., "search", "pagination").
    #[serde(default)]
    pub interaction_patterns: Vec<InteractionPattern>,

    // ── State Grammar: what indicates system state ──
    /// Named state indicators (e.g., "loading", "error", "captcha").
    #[serde(default)]
    pub state_indicators: Vec<StateIndicator>,

    // ── Navigation Grammar ──
    #[serde(default)]
    pub navigation: NavigationGrammar,

    // ── Intent Routes ──
    /// Maps user intents to grammar entries.
    #[serde(default)]
    pub intent_routes: Vec<IntentRoute>,

    /// User-pinned: never archive or compress this grammar.
    #[serde(default)]
    pub pinned: bool,

    /// Significance score [0.0, 1.0] — determines retention tier.
    #[serde(default)]
    pub significance: f32,

    /// Count of successful queries made using this grammar.
    #[serde(default)]
    pub query_success_count: u64,

    /// Count of failed queries made using this grammar.
    #[serde(default)]
    pub query_failure_count: u64,
}

fn default_status() -> GrammarStatus {
    GrammarStatus::Learning
}

impl SiteGrammar {
    /// Create a new grammar in Learning status.
    pub fn new(domain: impl Into<String>) -> Self {
        let now = now_secs();
        let domain = domain.into();
        Self {
            grammar_version: format!("{}-v1", chrono::Utc::now().format("%Y-Q%q")),
            domain,
            last_verified: None,
            structural_hash: None,
            status: GrammarStatus::Learning,
            created_at: now,
            updated_at: now,
            content_map: HashMap::new(),
            interaction_patterns: Vec::new(),
            state_indicators: Vec::new(),
            navigation: NavigationGrammar::default(),
            intent_routes: Vec::new(),
            pinned: false,
            significance: 0.0,
            query_success_count: 0,
            query_failure_count: 0,
        }
    }

    /// Success rate of queries using this grammar.
    pub fn success_rate(&self) -> f32 {
        let total = self.query_success_count + self.query_failure_count;
        if total == 0 {
            0.0
        } else {
            self.query_success_count as f32 / total as f32
        }
    }

    /// Average confidence across all content map entries.
    pub fn average_confidence(&self) -> f32 {
        if self.content_map.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.content_map.values().map(|e| e.confidence).sum();
        sum / self.content_map.len() as f32
    }

    /// Transition to Active status if confidence is high enough.
    pub fn maybe_activate(&mut self) {
        if self.status == GrammarStatus::Learning && self.average_confidence() > 0.8 {
            self.status = GrammarStatus::Active;
            self.updated_at = now_secs();
        }
    }

    /// Mark as drifted (site structure changed).
    pub fn mark_drifted(&mut self) {
        self.status = GrammarStatus::Drifted;
        self.updated_at = now_secs();
    }

    /// Record a successful query against this grammar.
    pub fn record_query_success(&mut self, content_key: &str) {
        self.query_success_count += 1;
        self.updated_at = now_secs();
        if let Some(entry) = self.content_map.get_mut(content_key) {
            entry.record_success();
        }
        self.maybe_activate();
    }

    /// Record a failed query against this grammar.
    pub fn record_query_failure(&mut self, content_key: &str) {
        self.query_failure_count += 1;
        self.updated_at = now_secs();
        if let Some(entry) = self.content_map.get_mut(content_key) {
            entry.record_failure();
        }
    }

    /// Add a content map entry.
    pub fn add_content(&mut self, name: impl Into<String>, selector: impl Into<String>) {
        self.content_map
            .insert(name.into(), ContentMapEntry::new(selector));
        self.updated_at = now_secs();
    }

    /// Add an intent route.
    pub fn add_intent_route(
        &mut self,
        intent: impl Into<String>,
        content_keys: Vec<String>,
        interaction: Option<String>,
    ) {
        self.intent_routes.push(IntentRoute {
            intent: intent.into(),
            content_keys,
            interaction,
        });
        self.updated_at = now_secs();
    }

    /// Find the intent route for a given intent name.
    pub fn route_intent(&self, intent: &str) -> Option<&IntentRoute> {
        self.intent_routes.iter().find(|r| r.intent == intent)
    }

    /// Get all selectors for a given intent (resolves route to actual selectors).
    pub fn selectors_for_intent(&self, intent: &str) -> Vec<&ContentMapEntry> {
        if let Some(route) = self.route_intent(intent) {
            route
                .content_keys
                .iter()
                .filter_map(|key| self.content_map.get(key.as_str()))
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// In-memory store for all site grammars.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrammarStore {
    /// Grammars indexed by domain.
    pub grammars: HashMap<String, SiteGrammar>,
}

impl GrammarStore {
    pub fn new() -> Self {
        Self {
            grammars: HashMap::new(),
        }
    }

    /// Get a grammar by domain.
    pub fn get(&self, domain: &str) -> Option<&SiteGrammar> {
        self.grammars.get(domain)
    }

    /// Get a mutable grammar by domain.
    pub fn get_mut(&mut self, domain: &str) -> Option<&mut SiteGrammar> {
        self.grammars.get_mut(domain)
    }

    /// Insert or replace a grammar.
    pub fn insert(&mut self, grammar: SiteGrammar) {
        self.grammars.insert(grammar.domain.clone(), grammar);
    }

    /// Check if a grammar exists for this domain.
    pub fn has(&self, domain: &str) -> bool {
        self.grammars.contains_key(domain)
    }

    /// Get all active grammars.
    pub fn active_grammars(&self) -> Vec<&SiteGrammar> {
        self.grammars
            .values()
            .filter(|g| g.status == GrammarStatus::Active)
            .collect()
    }

    /// Get all drifted grammars (need re-learning).
    pub fn drifted_grammars(&self) -> Vec<&SiteGrammar> {
        self.grammars
            .values()
            .filter(|g| g.status == GrammarStatus::Drifted)
            .collect()
    }

    /// Total number of grammars.
    pub fn count(&self) -> usize {
        self.grammars.len()
    }

    /// Remove a grammar by domain.
    pub fn remove(&mut self, domain: &str) -> Option<SiteGrammar> {
        self.grammars.remove(domain)
    }

    /// List all domains with grammars.
    pub fn domains(&self) -> Vec<&str> {
        self.grammars.keys().map(|s| s.as_str()).collect()
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grammar() {
        let g = SiteGrammar::new("amazon.com");
        assert_eq!(g.domain, "amazon.com");
        assert_eq!(g.status, GrammarStatus::Learning);
        assert_eq!(g.success_rate(), 0.0);
    }

    #[test]
    fn test_content_map_operations() {
        let mut g = SiteGrammar::new("amazon.com");
        g.add_content("product_price", ".a-price-whole");
        g.add_content("product_title", "#productTitle");

        assert_eq!(g.content_map.len(), 2);
        assert!(g.content_map.contains_key("product_price"));
    }

    #[test]
    fn test_confidence_tracking() {
        let mut entry = ContentMapEntry::new(".price");
        assert_eq!(entry.confidence, 0.5);

        entry.record_success();
        assert!((entry.confidence - 0.6).abs() < 0.01);

        entry.record_failure();
        assert!((entry.confidence - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_intent_routing() {
        let mut g = SiteGrammar::new("amazon.com");
        g.add_content("product_price", ".a-price-whole");
        g.add_intent_route("find_price", vec!["product_price".into()], None);

        let selectors = g.selectors_for_intent("find_price");
        assert_eq!(selectors.len(), 1);
        assert_eq!(selectors[0].selector, ".a-price-whole");

        let empty = g.selectors_for_intent("nonexistent");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_grammar_activation() {
        let mut g = SiteGrammar::new("test.com");
        g.add_content("title", "h1");

        // Boost confidence above 0.8
        for _ in 0..5 {
            g.record_query_success("title");
        }
        assert_eq!(g.status, GrammarStatus::Active);
    }

    #[test]
    fn test_grammar_store() {
        let mut store = GrammarStore::new();
        assert_eq!(store.count(), 0);

        let g = SiteGrammar::new("example.com");
        store.insert(g);
        assert!(store.has("example.com"));
        assert!(!store.has("other.com"));
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn test_grammar_drift() {
        let mut g = SiteGrammar::new("test.com");
        g.status = GrammarStatus::Active;
        g.mark_drifted();
        assert_eq!(g.status, GrammarStatus::Drifted);
    }
}
