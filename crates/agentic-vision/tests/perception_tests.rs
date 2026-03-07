//! Comprehensive edge-case and stress tests for the Perception Revolution.
//!
//! Covers: SiteGrammar, GrammarStore, IntentCache, DriftDetector, DomSnapshot,
//! SignificanceScorer, TokenBudget, PerceptionRouter, .avis v2 format.

use std::collections::HashMap;

use agentic_vision::perception::budget::{TokenBudget, TokenBudgetTier};
use agentic_vision::perception::cache::{
    CacheLookup, ContentVolatility, IntentCache, IntentCacheKey,
};
use agentic_vision::perception::dom::{AccessibilityNode, AccessibilityRole, DomSnapshot, NodeBounds};
use agentic_vision::perception::drift::{DriftDetector, DriftHistory, DriftSeverity};
use agentic_vision::perception::grammar::{
    ContentMapEntry, GrammarStatus, GrammarStore, InteractionPattern, NavigationGrammar,
    NavigationType, SelectorType, SiteGrammar, StateIndicator,
};
use agentic_vision::perception::router::{PerceptionLayer, PerceptionRouter};
use agentic_vision::perception::significance::{RetentionTier, SignificanceScore, SignificanceScorer};
use agentic_vision::perception::types::{
    ActionType, ContentScope, DataField, FallbackStrategy, FieldType, PerceptionIntent,
    PerceptionRequest,
};
use agentic_vision::storage::{AvisReader, AvisStoreV2, AvisWriter};
use agentic_vision::{
    CaptureSource, ObservationMeta, VisualMemoryStore, VisualObservation,
};

// ═══════════════════════════════════════════════════════════
// SECTION 1: GRAMMAR EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_grammar_empty_domain() {
    let g = SiteGrammar::new("");
    assert_eq!(g.domain, "");
    assert_eq!(g.status, GrammarStatus::Learning);
    assert_eq!(g.average_confidence(), 0.0);
}

#[test]
fn test_grammar_unicode_domain() {
    let g = SiteGrammar::new("日本語.jp");
    assert_eq!(g.domain, "日本語.jp");
}

#[test]
fn test_grammar_very_long_domain() {
    let domain = "a".repeat(1000);
    let g = SiteGrammar::new(&domain);
    assert_eq!(g.domain.len(), 1000);
}

#[test]
fn test_grammar_duplicate_content_keys() {
    let mut g = SiteGrammar::new("test.com");
    g.add_content("price", ".old-selector");
    g.add_content("price", ".new-selector");
    // Second insert should overwrite
    assert_eq!(g.content_map.len(), 1);
    assert_eq!(g.content_map.get("price").unwrap().selector, ".new-selector");
}

#[test]
fn test_grammar_confidence_bounds() {
    let mut entry = ContentMapEntry::new(".test");

    // Boost to max
    for _ in 0..20 {
        entry.record_success();
    }
    assert_eq!(entry.confidence, 1.0); // Should cap at 1.0

    // Drop to zero
    for _ in 0..20 {
        entry.record_failure();
    }
    assert_eq!(entry.confidence, 0.0); // Should floor at 0.0
}

#[test]
fn test_grammar_success_rate_no_queries() {
    let g = SiteGrammar::new("test.com");
    assert_eq!(g.success_rate(), 0.0);
}

#[test]
fn test_grammar_success_rate_all_success() {
    let mut g = SiteGrammar::new("test.com");
    g.query_success_count = 100;
    g.query_failure_count = 0;
    assert_eq!(g.success_rate(), 1.0);
}

#[test]
fn test_grammar_success_rate_all_failure() {
    let mut g = SiteGrammar::new("test.com");
    g.query_success_count = 0;
    g.query_failure_count = 100;
    assert_eq!(g.success_rate(), 0.0);
}

#[test]
fn test_grammar_activation_threshold() {
    let mut g = SiteGrammar::new("test.com");
    g.add_content("a", "sel-a");

    // Confidence 0.5 (default) — should stay Learning
    g.maybe_activate();
    assert_eq!(g.status, GrammarStatus::Learning);

    // Boost to 0.9 — should activate
    g.content_map.get_mut("a").unwrap().confidence = 0.9;
    g.maybe_activate();
    assert_eq!(g.status, GrammarStatus::Active);
}

#[test]
fn test_grammar_activation_requires_learning_status() {
    let mut g = SiteGrammar::new("test.com");
    g.add_content("a", "sel-a");
    g.content_map.get_mut("a").unwrap().confidence = 0.95;
    g.status = GrammarStatus::Drifted;

    g.maybe_activate();
    // Should NOT activate from Drifted status
    assert_eq!(g.status, GrammarStatus::Drifted);
}

#[test]
fn test_grammar_intent_route_not_found() {
    let g = SiteGrammar::new("test.com");
    assert!(g.route_intent("nonexistent").is_none());
    assert!(g.selectors_for_intent("nonexistent").is_empty());
}

#[test]
fn test_grammar_intent_route_with_missing_content_key() {
    let mut g = SiteGrammar::new("test.com");
    // Route references keys that don't exist in content_map
    g.add_intent_route("find_price", vec!["missing_key".into()], None);
    let selectors = g.selectors_for_intent("find_price");
    assert!(selectors.is_empty());
}

#[test]
fn test_grammar_multiple_intent_routes() {
    let mut g = SiteGrammar::new("amazon.com");
    g.add_content("price", ".a-price");
    g.add_content("title", "#title");
    g.add_content("rating", "#rating");
    g.add_intent_route("find_price", vec!["price".into()], None);
    g.add_intent_route("find_details", vec!["title".into(), "rating".into()], None);

    assert_eq!(g.selectors_for_intent("find_price").len(), 1);
    assert_eq!(g.selectors_for_intent("find_details").len(), 2);
}

#[test]
fn test_grammar_record_query_for_nonexistent_key() {
    let mut g = SiteGrammar::new("test.com");
    // Should not panic even if key doesn't exist
    g.record_query_success("nonexistent");
    g.record_query_failure("nonexistent");
    assert_eq!(g.query_success_count, 1);
    assert_eq!(g.query_failure_count, 1);
}

#[test]
fn test_grammar_selector_types() {
    let mut entry = ContentMapEntry::new("//div[@class='price']");
    entry.selector_type = SelectorType::Xpath;
    assert_eq!(entry.selector_type, SelectorType::Xpath);

    let mut entry2 = ContentMapEntry::new("[aria-label='price']");
    entry2.selector_type = SelectorType::Aria;
    assert_eq!(entry2.selector_type, SelectorType::Aria);
}

#[test]
fn test_grammar_fallback_selectors() {
    let mut entry = ContentMapEntry::new(".primary");
    entry.fallback_selectors = vec![".fallback1".into(), ".fallback2".into()];
    assert_eq!(entry.fallback_selectors.len(), 2);
}

#[test]
fn test_grammar_pinning() {
    let mut g = SiteGrammar::new("test.com");
    assert!(!g.pinned);
    g.pinned = true;
    assert!(g.pinned);
}

#[test]
fn test_grammar_serialization_roundtrip() {
    let mut g = SiteGrammar::new("complex.example.com");
    g.add_content("price", ".price-tag");
    g.add_content("title", "h1.main-title");
    g.add_intent_route("find_price", vec!["price".into()], None);
    g.interaction_patterns.push(InteractionPattern {
        name: "search".into(),
        steps: [("input".into(), "#search-box".into())].into_iter().collect(),
        success_indicator: Some(".results-loaded".into()),
    });
    g.state_indicators.push(StateIndicator {
        state_name: "loading".into(),
        selector: ".spinner".into(),
    });
    g.navigation = NavigationGrammar {
        navigation_type: NavigationType::Spa,
        js_required: true,
        spa_navigation: true,
        back_button_safe: false,
        session_state: Some("cookie".into()),
    };
    g.pinned = true;
    g.significance = 0.85;

    let json = serde_json::to_string(&g).unwrap();
    let deserialized: SiteGrammar = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.domain, "complex.example.com");
    assert_eq!(deserialized.content_map.len(), 2);
    assert_eq!(deserialized.intent_routes.len(), 1);
    assert_eq!(deserialized.interaction_patterns.len(), 1);
    assert_eq!(deserialized.state_indicators.len(), 1);
    assert!(deserialized.pinned);
    assert!((deserialized.significance - 0.85).abs() < 0.001);
    assert!(deserialized.navigation.js_required);
}

// ═══════════════════════════════════════════════════════════
// SECTION 2: GRAMMAR STORE EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_grammar_store_empty() {
    let store = GrammarStore::new();
    assert_eq!(store.count(), 0);
    assert!(store.get("anything").is_none());
    assert!(!store.has("anything"));
    assert!(store.active_grammars().is_empty());
    assert!(store.drifted_grammars().is_empty());
    assert!(store.domains().is_empty());
}

#[test]
fn test_grammar_store_insert_and_replace() {
    let mut store = GrammarStore::new();
    let mut g1 = SiteGrammar::new("test.com");
    g1.add_content("field_a", ".sel-a");
    store.insert(g1);

    let mut g2 = SiteGrammar::new("test.com");
    g2.add_content("field_b", ".sel-b");
    store.insert(g2);

    // Should have replaced
    assert_eq!(store.count(), 1);
    let g = store.get("test.com").unwrap();
    assert!(g.content_map.contains_key("field_b"));
    assert!(!g.content_map.contains_key("field_a"));
}

#[test]
fn test_grammar_store_remove() {
    let mut store = GrammarStore::new();
    store.insert(SiteGrammar::new("test.com"));
    assert!(store.has("test.com"));

    let removed = store.remove("test.com");
    assert!(removed.is_some());
    assert!(!store.has("test.com"));

    let removed_again = store.remove("test.com");
    assert!(removed_again.is_none());
}

#[test]
fn test_grammar_store_filter_by_status() {
    let mut store = GrammarStore::new();

    let mut g1 = SiteGrammar::new("active.com");
    g1.status = GrammarStatus::Active;
    store.insert(g1);

    let mut g2 = SiteGrammar::new("drifted.com");
    g2.status = GrammarStatus::Drifted;
    store.insert(g2);

    let mut g3 = SiteGrammar::new("learning.com");
    g3.status = GrammarStatus::Learning;
    store.insert(g3);

    assert_eq!(store.active_grammars().len(), 1);
    assert_eq!(store.drifted_grammars().len(), 1);
}

// ═══════════════════════════════════════════════════════════
// SECTION 3: INTENT CACHE EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_cache_empty_lookup() {
    let mut cache = IntentCache::new();
    let key = IntentCacheKey::new("https://x.com", "price", "s1", "c1");
    assert!(matches!(cache.lookup(&key), CacheLookup::Miss));
}

#[test]
fn test_cache_hit_increments_stats() {
    let mut cache = IntentCache::new();
    let key = IntentCacheKey::new("https://x.com", "price", "s1", "c1");
    cache.insert(
        key.clone(),
        serde_json::json!({"price": "100"}),
        ContentVolatility::Static,
        2000,
    );

    let stats_before = cache.stats();
    assert_eq!(stats_before.total_hits, 0);

    let _ = cache.lookup(&key);
    let stats_after = cache.stats();
    assert_eq!(stats_after.total_hits, 1);
}

#[test]
fn test_cache_miss_increments_stats() {
    let mut cache = IntentCache::new();
    let key = IntentCacheKey::new("https://x.com", "price", "s1", "c1");
    let _ = cache.lookup(&key);
    assert_eq!(cache.stats().total_misses, 1);
}

#[test]
fn test_cache_url_normalization() {
    let key1 = IntentCacheKey::new("HTTPS://EXAMPLE.COM/Page/", "intent", "s", "c");
    let key2 = IntentCacheKey::new("https://example.com/page", "intent", "s", "c");
    assert_eq!(key1.url_normalized, key2.url_normalized);
}

#[test]
fn test_cache_strips_utm_params() {
    let key = IntentCacheKey::new(
        "https://example.com/page?utm_source=twitter&id=123&utm_campaign=test",
        "intent",
        "s",
        "c",
    );
    assert_eq!(key.url_normalized, "https://example.com/page?id=123");
}

#[test]
fn test_cache_domain_invalidation() {
    let mut cache = IntentCache::new();
    cache.insert(
        IntentCacheKey::new("https://a.example.com/p1", "i1", "s", "c"),
        serde_json::json!({}),
        ContentVolatility::Static,
        100,
    );
    cache.insert(
        IntentCacheKey::new("https://b.example.com/p2", "i2", "s", "c"),
        serde_json::json!({}),
        ContentVolatility::Static,
        100,
    );
    cache.insert(
        IntentCacheKey::new("https://other.com/p3", "i3", "s", "c"),
        serde_json::json!({}),
        ContentVolatility::Static,
        100,
    );
    assert_eq!(cache.len(), 3);

    cache.invalidate_domain("example.com");
    assert_eq!(cache.len(), 1); // only other.com remains
}

#[test]
fn test_cache_replaces_same_url_intent() {
    let mut cache = IntentCache::new();
    let key1 = IntentCacheKey::new("https://x.com/p", "price", "s1", "c1");
    cache.insert(key1, serde_json::json!({"v": 1}), ContentVolatility::Dynamic, 100);

    let key2 = IntentCacheKey::new("https://x.com/p", "price", "s1", "c2");
    cache.insert(key2, serde_json::json!({"v": 2}), ContentVolatility::Dynamic, 100);

    // Old entry should be replaced (same url+intent, different content hash)
    assert_eq!(cache.len(), 1);
}

#[test]
fn test_cache_volatility_ttl() {
    assert_eq!(ContentVolatility::Dynamic.ttl_secs(), 3600);
    assert_eq!(ContentVolatility::SemiStatic.ttl_secs(), 86400);
    assert_eq!(ContentVolatility::Static.ttl_secs(), 604800);
    assert_eq!(ContentVolatility::Pinned.ttl_secs(), u64::MAX);
}

#[test]
fn test_cache_eviction_on_max() {
    let mut cache = IntentCache::with_max_entries(5);
    for i in 0..10 {
        cache.insert(
            IntentCacheKey::new(format!("https://site-{i}.com"), "intent", "s", "c"),
            serde_json::json!({}),
            ContentVolatility::Static,
            10,
        );
    }
    assert!(cache.len() <= 5);
}

// ═══════════════════════════════════════════════════════════
// SECTION 4: DRIFT DETECTION EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_drift_identical_hashes() {
    let result = DriftDetector::detect("x.com", "hash1", "hash1", "v1", vec![], vec![]);
    assert!(result.is_none());
}

#[test]
fn test_drift_empty_hashes() {
    let result = DriftDetector::detect("x.com", "", "", "v1", vec![], vec![]);
    assert!(result.is_none()); // empty == empty
}

#[test]
fn test_drift_severity_no_selectors() {
    // All broken / working lists empty: should be minor (0/0 = 0% broken)
    let event = DriftDetector::detect("x.com", "a", "b", "v1", vec![], vec![]).unwrap();
    assert_eq!(event.severity, DriftSeverity::Minor);
}

#[test]
fn test_drift_severity_100_percent_broken() {
    let event = DriftDetector::detect(
        "x.com",
        "a",
        "b",
        "v1",
        vec!["a".into(), "b".into(), "c".into()],
        vec![],
    )
    .unwrap();
    assert_eq!(event.severity, DriftSeverity::Major);
}

#[test]
fn test_drift_severity_boundaries() {
    // Exactly 20% broken → Minor (not Moderate)
    let event = DriftDetector::detect(
        "x.com",
        "a",
        "b",
        "v1",
        vec!["broken".into()],
        vec!["w1".into(), "w2".into(), "w3".into(), "w4".into()],
    )
    .unwrap();
    assert_eq!(event.severity, DriftSeverity::Minor); // 1/5 = 20%

    // 21% → Moderate
    let event = DriftDetector::detect(
        "x.com",
        "a",
        "b",
        "v1",
        vec!["b1".into(), "b2".into()],
        vec!["w1".into(), "w2".into(), "w3".into(), "w4".into(), "w5".into(), "w6".into(), "w7".into()],
    )
    .unwrap();
    // 2/9 ≈ 22% → Moderate
    assert_eq!(event.severity, DriftSeverity::Moderate);
}

#[test]
fn test_drift_relearn_cost() {
    assert!(DriftDetector::estimated_relearn_cost(DriftSeverity::Minor) < DriftDetector::estimated_relearn_cost(DriftSeverity::Moderate));
    assert!(DriftDetector::estimated_relearn_cost(DriftSeverity::Moderate) < DriftDetector::estimated_relearn_cost(DriftSeverity::Major));
}

#[test]
fn test_drift_history_frequency_single_event() {
    let mut h = DriftHistory::new();
    h.record(make_drift_event("x.com", 1000));
    assert_eq!(h.drift_frequency("x.com"), 0.0); // Need ≥2 events
}

#[test]
fn test_drift_history_frequency_same_timestamp() {
    let mut h = DriftHistory::new();
    h.record(make_drift_event("x.com", 1000));
    h.record(make_drift_event("x.com", 1000));
    assert_eq!(h.drift_frequency("x.com"), 0.0); // 0 span
}

fn make_drift_event(domain: &str, detected_at: u64) -> agentic_vision::perception::drift::DriftEvent {
    agentic_vision::perception::drift::DriftEvent {
        domain: domain.into(),
        old_hash: "a".into(),
        new_hash: "b".into(),
        severity: DriftSeverity::Minor,
        broken_selectors: vec![],
        working_selectors: vec![],
        detected_at,
        relearn_triggered: false,
        old_version: "v1".into(),
        new_version: None,
    }
}

// ═══════════════════════════════════════════════════════════
// SECTION 5: DOM SNAPSHOT EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_dom_snapshot_empty() {
    let snap = DomSnapshot::new("https://x.com", "x.com");
    assert!(snap.nodes.is_empty());
    assert_eq!(snap.text_content(), "");
    assert_eq!(snap.interactive_elements().len(), 0);
}

#[test]
fn test_dom_snapshot_structural_hash_deterministic() {
    let mut snap1 = DomSnapshot::new("https://x.com", "x.com");
    snap1.add_node(make_a11y_node(1, AccessibilityRole::Main, "main"));
    snap1.compute_structural_hash();

    let mut snap2 = DomSnapshot::new("https://x.com", "x.com");
    snap2.add_node(make_a11y_node(1, AccessibilityRole::Main, "main"));
    snap2.compute_structural_hash();

    assert_eq!(snap1.structural_hash, snap2.structural_hash);
}

#[test]
fn test_dom_snapshot_structural_hash_differs_on_change() {
    let mut snap1 = DomSnapshot::new("https://x.com", "x.com");
    snap1.add_node(make_a11y_node(1, AccessibilityRole::Main, "main"));
    snap1.compute_structural_hash();

    let mut snap2 = DomSnapshot::new("https://x.com", "x.com");
    snap2.add_node(make_a11y_node(1, AccessibilityRole::Navigation, "nav"));
    snap2.compute_structural_hash();

    assert_ne!(snap1.structural_hash, snap2.structural_hash);
}

#[test]
fn test_dom_snapshot_content_hash() {
    let mut snap = DomSnapshot::new("https://x.com", "x.com");
    snap.add_node(AccessibilityNode {
        node_id: 1,
        role: AccessibilityRole::Heading,
        name: Some("Hello World".into()),
        description: None,
        value: None,
        selector: Some("h1".into()),
        interactive: false,
        visible: true,
        bounds: None,
        attributes: HashMap::new(),
        children: vec![],
    });
    snap.compute_content_hash();
    assert!(snap.content_hash.starts_with("blake3:"));
}

#[test]
fn test_dom_snapshot_find_by_role() {
    let mut snap = DomSnapshot::new("https://x.com", "x.com");
    snap.add_node(make_a11y_node(1, AccessibilityRole::Button, "button#a"));
    snap.add_node(make_a11y_node(2, AccessibilityRole::Button, "button#b"));
    snap.add_node(make_a11y_node(3, AccessibilityRole::Link, "a.link"));

    assert_eq!(snap.find_by_role(&AccessibilityRole::Button).len(), 2);
    assert_eq!(snap.find_by_role(&AccessibilityRole::Link).len(), 1);
    assert_eq!(snap.find_by_role(&AccessibilityRole::Heading).len(), 0);
}

#[test]
fn test_dom_snapshot_selector_pattern() {
    let mut snap = DomSnapshot::new("https://x.com", "x.com");
    snap.add_node(make_a11y_node(1, AccessibilityRole::Button, "button.primary"));
    snap.add_node(make_a11y_node(2, AccessibilityRole::Button, "button.secondary"));
    snap.add_node(make_a11y_node(3, AccessibilityRole::Link, "a.nav-link"));

    assert_eq!(snap.find_by_selector_pattern("button").len(), 2);
    assert_eq!(snap.find_by_selector_pattern("primary").len(), 1);
    assert_eq!(snap.find_by_selector_pattern("nonexistent").len(), 0);
}

#[test]
fn test_dom_snapshot_node_bounds() {
    let mut snap = DomSnapshot::new("https://x.com", "x.com");
    let mut node = make_a11y_node(1, AccessibilityRole::Button, "button");
    node.bounds = Some(NodeBounds {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
    });
    snap.add_node(node);

    let n = snap.nodes.get(&1).unwrap();
    assert!(n.bounds.is_some());
    let b = n.bounds.unwrap();
    assert!((b.width - 100.0).abs() < 0.01);
}

fn make_a11y_node(id: u64, role: AccessibilityRole, selector: &str) -> AccessibilityNode {
    AccessibilityNode {
        node_id: id,
        role,
        name: None,
        description: None,
        value: None,
        selector: Some(selector.into()),
        interactive: false,
        visible: true,
        bounds: None,
        attributes: HashMap::new(),
        children: vec![],
    }
}

// ═══════════════════════════════════════════════════════════
// SECTION 6: TOKEN BUDGET EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_budget_zero_consumption() {
    let budget = TokenBudget::surgical();
    assert_eq!(budget.tokens_used, 0);
    assert_eq!(budget.remaining(), 50);
    assert!(!budget.is_exhausted());
}

#[test]
fn test_budget_exact_limit() {
    let mut budget = TokenBudget::surgical(); // max 50
    assert!(budget.consume(50));
    assert!(budget.is_exhausted());
    assert_eq!(budget.remaining(), 0);
}

#[test]
fn test_budget_overflow_protection() {
    let mut budget = TokenBudget::surgical(); // max 50
    budget.consume(30);
    assert!(!budget.consume(30)); // 60 > 50
    assert!(budget.is_exhausted());
}

#[test]
fn test_budget_custom_max_overrides_tier() {
    let budget = TokenBudget {
        tier: TokenBudgetTier::Surgical,
        max_tokens: Some(1000),
        tokens_used: 0,
    };
    assert_eq!(budget.effective_max(), 1000);
    assert_eq!(budget.remaining(), 1000);
}

#[test]
fn test_budget_all_tiers() {
    let tiers = [
        (TokenBudgetTier::Surgical, 50),
        (TokenBudgetTier::Focused, 300),
        (TokenBudgetTier::Contextual, 800),
        (TokenBudgetTier::Visual, 2000),
        (TokenBudgetTier::FullPage, 5000),
    ];
    for (tier, expected_max) in &tiers {
        assert_eq!(tier.max_tokens(), *expected_max, "Tier {:?} failed", tier);
    }
}

// ═══════════════════════════════════════════════════════════
// SECTION 7: SIGNIFICANCE SCORING EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_significance_tier_boundaries() {
    assert_eq!(SignificanceScore::tier_from_score(0.0), RetentionTier::Archive);
    assert_eq!(SignificanceScore::tier_from_score(0.2), RetentionTier::Archive);
    assert_eq!(SignificanceScore::tier_from_score(0.201), RetentionTier::Cold);
    assert_eq!(SignificanceScore::tier_from_score(0.4), RetentionTier::Cold);
    assert_eq!(SignificanceScore::tier_from_score(0.401), RetentionTier::Standard);
    assert_eq!(SignificanceScore::tier_from_score(0.7), RetentionTier::Standard);
    assert_eq!(SignificanceScore::tier_from_score(0.701), RetentionTier::Active);
    assert_eq!(SignificanceScore::tier_from_score(1.0), RetentionTier::Active);
}

#[test]
fn test_significance_zero_max_usage() {
    // max_usage is clamped to 1
    let scorer = SignificanceScorer::new(0);
    let g = SiteGrammar::new("test.com");
    let score = scorer.score(&g, 0.0);
    assert!(score.score >= 0.0);
}

#[test]
fn test_significance_high_usage_high_importance() {
    let mut g = SiteGrammar::new("amazon.com");
    g.query_success_count = 1000;
    g.query_failure_count = 10;
    for i in 0..15 {
        g.add_content(format!("field_{i}"), format!(".sel-{i}"));
    }

    let scorer = SignificanceScorer::new(1000);
    let score = scorer.score(&g, 0.95);
    assert!(score.score > 0.7, "High-value grammar should be Active tier, got {}", score.score);
    assert_eq!(score.tier, RetentionTier::Active);
}

// ═══════════════════════════════════════════════════════════
// SECTION 8: PERCEPTION ROUTER EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_router_extract_data_with_grammar() {
    let mut store = GrammarStore::new();
    let mut g = SiteGrammar::new("amazon.com");
    g.add_content("price", ".a-price");
    store.insert(g);

    let mut cache = IntentCache::new();
    let req = PerceptionRequest::extract_data(
        "https://amazon.com/dp/X",
        vec![DataField { name: "price".into(), field_type: FieldType::Currency }],
    );

    let decision = PerceptionRouter::route(&req, &store, &mut cache);
    assert_eq!(decision.primary_layer, PerceptionLayer::GrammarLookup);
    assert!(!decision.needs_screenshot);
}

#[test]
fn test_router_extract_data_without_grammar() {
    let store = GrammarStore::new();
    let mut cache = IntentCache::new();
    let req = PerceptionRequest::extract_data(
        "https://unknown.com",
        vec![DataField { name: "title".into(), field_type: FieldType::Text }],
    );

    let decision = PerceptionRouter::route(&req, &store, &mut cache);
    assert_eq!(decision.primary_layer, PerceptionLayer::DomExtraction);
}

#[test]
fn test_router_visual_intent_always_screenshot() {
    let store = GrammarStore::new();
    let mut cache = IntentCache::new();

    for intent in [
        PerceptionIntent::AnalyzeVisual,
        PerceptionIntent::ReadDocument,
        PerceptionIntent::VerifyCaptcha,
    ] {
        let req = PerceptionRequest {
            intent,
            url: Some("https://x.com".into()),
            domain: Some("x.com".into()),
            budget: TokenBudget::visual(),
            fallback: FallbackStrategy::Fail,
        };
        let decision = PerceptionRouter::route(&req, &store, &mut cache);
        assert_eq!(decision.primary_layer, PerceptionLayer::ScopedScreenshot);
        assert!(decision.needs_screenshot);
    }
}

#[test]
fn test_router_monitor_uses_delta() {
    let store = GrammarStore::new();
    let mut cache = IntentCache::new();
    let req = PerceptionRequest::monitor("https://x.com", Some("hash123".into()));
    let decision = PerceptionRouter::route(&req, &store, &mut cache);
    assert_eq!(decision.primary_layer, PerceptionLayer::DeltaVision);
}

#[test]
fn test_router_no_url() {
    let store = GrammarStore::new();
    let mut cache = IntentCache::new();
    let req = PerceptionRequest {
        intent: PerceptionIntent::ExtractData {
            fields: vec![DataField { name: "x".into(), field_type: FieldType::Text }],
        },
        url: None,
        domain: None,
        budget: TokenBudget::surgical(),
        fallback: FallbackStrategy::Escalate,
    };
    let decision = PerceptionRouter::route(&req, &store, &mut cache);
    // No URL → no grammar lookup → DOM extraction
    assert_eq!(decision.primary_layer, PerceptionLayer::DomExtraction);
}

#[test]
fn test_perception_layer_properties() {
    assert_eq!(PerceptionLayer::DomExtraction.index(), 0);
    assert_eq!(PerceptionLayer::ScopedScreenshot.index(), 4);

    // Grammar lookup should be cheapest
    assert_eq!(PerceptionLayer::GrammarLookup.typical_tokens(), 0);
    // Scoped screenshot should be most expensive of the non-full layers
    assert!(PerceptionLayer::ScopedScreenshot.typical_tokens() > PerceptionLayer::DomExtraction.typical_tokens());
}

// ═══════════════════════════════════════════════════════════
// SECTION 9: .AVIS V2 FORMAT EDGE CASES
// ═══════════════════════════════════════════════════════════

fn make_test_obs(id: u64) -> VisualObservation {
    VisualObservation {
        id,
        timestamp: 1708345678,
        session_id: 1,
        source: CaptureSource::File { path: "/test.png".into() },
        embedding: vec![0.1, 0.2, 0.3],
        thumbnail: vec![0xFF, 0xD8, 0xFF],
        metadata: ObservationMeta {
            width: 512,
            height: 512,
            original_width: 1920,
            original_height: 1080,
            labels: vec!["test".into()],
            description: Some("Test".into()),
            quality_score: 0.85,
        },
        memory_link: None,
    }
}

#[test]
fn test_v2_empty_roundtrip() {
    let v2 = AvisStoreV2::new(512);
    let mut buf = Vec::new();
    AvisWriter::write_v2_to(&v2, &mut buf).unwrap();
    let loaded = AvisReader::read_v2_from(&mut &buf[..]).unwrap();
    assert_eq!(loaded.store.count(), 0);
    assert_eq!(loaded.grammar_store.count(), 0);
    assert!(loaded.intent_cache.is_empty());
}

#[test]
fn test_v2_with_observations_and_grammars() {
    let mut v2 = AvisStoreV2::new(512);
    v2.store.add(make_test_obs(0));
    v2.store.add(make_test_obs(0));

    let mut g = SiteGrammar::new("test.com");
    g.add_content("title", "h1");
    g.add_content("price", ".price");
    g.add_intent_route("find_price", vec!["price".into()], None);
    v2.grammar_store.insert(g);

    let mut buf = Vec::new();
    AvisWriter::write_v2_to(&v2, &mut buf).unwrap();
    let loaded = AvisReader::read_v2_from(&mut &buf[..]).unwrap();

    assert_eq!(loaded.store.count(), 2);
    assert!(loaded.grammar_store.has("test.com"));
    let lg = loaded.grammar_store.get("test.com").unwrap();
    assert_eq!(lg.content_map.len(), 2);
    assert_eq!(lg.intent_routes.len(), 1);
}

#[test]
fn test_v2_with_cache_and_drift() {
    let mut v2 = AvisStoreV2::new(512);

    v2.intent_cache.insert(
        IntentCacheKey::new("https://x.com", "price", "s", "c"),
        serde_json::json!({"price": "$99"}),
        ContentVolatility::Dynamic,
        2000,
    );

    v2.drift_history.record(make_drift_event("x.com", 1000));

    let mut buf = Vec::new();
    AvisWriter::write_v2_to(&v2, &mut buf).unwrap();
    let loaded = AvisReader::read_v2_from(&mut &buf[..]).unwrap();

    assert_eq!(loaded.intent_cache.len(), 1);
    assert_eq!(loaded.drift_history.count_for_domain("x.com"), 1);
}

#[test]
fn test_v2_backward_compat_reads_v1() {
    // Write a v1 file manually (old format: version=1, no grammar fields)
    let store = VisualMemoryStore::new(512);
    let payload = serde_json::to_vec(&serde_json::json!({
        "observations": [],
        "embedding_dim": 512,
        "next_id": 1,
        "session_count": 0,
        "created_at": 1000,
        "updated_at": 1000,
    })).unwrap();

    let mut buf = Vec::new();
    let mut header = [0u8; 64];
    header[0..4].copy_from_slice(&0x41564953u32.to_le_bytes()); // AVIS magic
    header[4..6].copy_from_slice(&1u16.to_le_bytes()); // version 1
    header[16..20].copy_from_slice(&512u32.to_le_bytes()); // embedding dim
    header[40..48].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    buf.extend_from_slice(&header);
    buf.extend_from_slice(&payload);

    let loaded = AvisReader::read_v2_from(&mut &buf[..]).unwrap();
    assert_eq!(loaded.store.count(), 0);
    assert_eq!(loaded.grammar_store.count(), 0); // default empty
    assert!(loaded.intent_cache.is_empty());
}

#[test]
fn test_v2_file_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("roundtrip.avis");

    let mut v2 = AvisStoreV2::new(512);
    v2.store.add(make_test_obs(0));
    let mut g = SiteGrammar::new("github.com");
    g.add_content("repo", ".repo-list");
    g.pinned = true;
    v2.grammar_store.insert(g);

    AvisWriter::write_v2_to_file(&v2, &path).unwrap();
    let loaded = AvisReader::read_v2_from_file(&path).unwrap();

    assert_eq!(loaded.store.count(), 1);
    assert!(loaded.grammar_store.get("github.com").unwrap().pinned);
}

#[test]
fn test_v2_invalid_magic_rejected() {
    let mut buf = [0u8; 128];
    buf[0..4].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
    let result = AvisReader::read_v2_from(&mut &buf[..]);
    assert!(result.is_err());
}

#[test]
fn test_v2_unsupported_version_rejected() {
    let mut buf = [0u8; 128];
    buf[0..4].copy_from_slice(&0x41564953u32.to_le_bytes());
    buf[4..6].copy_from_slice(&99u16.to_le_bytes()); // version 99
    let result = AvisReader::read_v2_from(&mut &buf[..]);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════
// SECTION 10: STRESS TESTS
// ═══════════════════════════════════════════════════════════

#[test]
fn test_stress_1000_grammars() {
    let mut store = GrammarStore::new();
    for i in 0..1000 {
        let mut g = SiteGrammar::new(format!("site-{i}.com"));
        for j in 0..10 {
            g.add_content(format!("field_{j}"), format!(".sel-{j}"));
        }
        g.add_intent_route("find_data", vec!["field_0".into(), "field_1".into()], None);
        store.insert(g);
    }
    assert_eq!(store.count(), 1000);

    // All should be findable
    for i in 0..1000 {
        assert!(store.has(&format!("site-{i}.com")));
    }
}

#[test]
fn test_stress_grammar_rapid_confidence_updates() {
    let mut g = SiteGrammar::new("test.com");
    g.add_content("field", ".selector");

    // 10,000 rapid success/failure alternations
    for i in 0..10_000 {
        if i % 3 == 0 {
            g.record_query_failure("field");
        } else {
            g.record_query_success("field");
        }
    }

    let entry = g.content_map.get("field").unwrap();
    assert!(entry.confidence >= 0.0 && entry.confidence <= 1.0);
    assert_eq!(g.query_success_count + g.query_failure_count, 10_000);
}

#[test]
fn test_stress_cache_10000_entries() {
    let mut cache = IntentCache::with_max_entries(10_000);
    for i in 0..10_000 {
        cache.insert(
            IntentCacheKey::new(format!("https://site-{i}.com/page"), "intent", "s", "c"),
            serde_json::json!({"i": i}),
            ContentVolatility::Static,
            100,
        );
    }
    assert_eq!(cache.len(), 10_000);

    // All should be hittable
    let mut hits = 0;
    for i in 0..10_000 {
        let key = IntentCacheKey::new(format!("https://site-{i}.com/page"), "intent", "s", "c");
        if matches!(cache.lookup(&key), CacheLookup::Hit(_)) {
            hits += 1;
        }
    }
    assert_eq!(hits, 10_000);
}

#[test]
fn test_stress_cache_eviction() {
    let mut cache = IntentCache::with_max_entries(100);
    for i in 0..500 {
        cache.insert(
            IntentCacheKey::new(format!("https://site-{i}.com"), "intent", "s", "c"),
            serde_json::json!({}),
            ContentVolatility::Static,
            10,
        );
    }
    assert!(cache.len() <= 100);
}

#[test]
fn test_stress_dom_snapshot_1000_nodes() {
    let mut snap = DomSnapshot::new("https://large-page.com", "large-page.com");
    for i in 0..1000 {
        snap.add_node(AccessibilityNode {
            node_id: i,
            role: if i % 5 == 0 { AccessibilityRole::Button } else { AccessibilityRole::Region },
            name: Some(format!("Node {i}")),
            description: None,
            value: if i % 10 == 0 { Some(format!("val-{i}")) } else { None },
            selector: Some(format!("div.node-{i}")),
            interactive: i % 5 == 0,
            visible: true,
            bounds: Some(NodeBounds { x: i as f32, y: 0.0, width: 100.0, height: 50.0 }),
            attributes: HashMap::new(),
            children: vec![],
        });
    }

    assert_eq!(snap.nodes.len(), 1000);
    assert_eq!(snap.find_by_role(&AccessibilityRole::Button).len(), 200);
    assert_eq!(snap.interactive_elements().len(), 200);

    let text = snap.text_content();
    assert!(text.contains("Node 0"));
    assert!(text.contains("Node 999"));

    snap.compute_structural_hash();
    assert!(snap.structural_hash.starts_with("blake3:"));

    snap.compute_content_hash();
    assert!(snap.content_hash.starts_with("blake3:"));
}

#[test]
fn test_stress_drift_history_1000_events() {
    let mut history = DriftHistory::new();
    for i in 0..1000 {
        history.record(make_drift_event(&format!("site-{}.com", i % 10), i * 86400));
    }

    assert_eq!(history.count_for_domain("site-0.com"), 100);
    assert_eq!(history.count_for_domain("site-9.com"), 100);
    assert!(history.latest("site-0.com").is_some());
}

#[test]
fn test_stress_v2_roundtrip_large() {
    let mut v2 = AvisStoreV2::new(512);

    // 100 observations
    for _ in 0..100 {
        v2.store.add(make_test_obs(0));
    }

    // 50 grammars with 20 content entries each
    for i in 0..50 {
        let mut g = SiteGrammar::new(format!("site-{i}.example.com"));
        for j in 0..20 {
            g.add_content(format!("field_{j}"), format!(".selector-{j}"));
        }
        g.add_intent_route("default", vec!["field_0".into()], None);
        g.query_success_count = 100;
        v2.grammar_store.insert(g);
    }

    // 200 cache entries
    for i in 0..200 {
        v2.intent_cache.insert(
            IntentCacheKey::new(format!("https://site-{i}.com/p"), "intent", "s", "c"),
            serde_json::json!({"data": i}),
            ContentVolatility::Static,
            100,
        );
    }

    // 30 drift events
    for i in 0..30 {
        v2.drift_history.record(make_drift_event(&format!("site-{i}.com"), i * 1000));
    }

    let mut buf = Vec::new();
    AvisWriter::write_v2_to(&v2, &mut buf).unwrap();

    let loaded = AvisReader::read_v2_from(&mut &buf[..]).unwrap();
    assert_eq!(loaded.store.count(), 100);
    assert_eq!(loaded.grammar_store.count(), 50);
    assert_eq!(loaded.intent_cache.len(), 200);
    assert_eq!(loaded.drift_history.count_for_domain("site-0.com"), 1);

    // Verify grammar content survived
    let g = loaded.grammar_store.get("site-25.example.com").unwrap();
    assert_eq!(g.content_map.len(), 20);
    assert_eq!(g.query_success_count, 100);
}

#[test]
fn test_stress_router_1000_routing_decisions() {
    let mut store = GrammarStore::new();
    for i in 0..100 {
        let mut g = SiteGrammar::new(format!("site-{i}.com"));
        g.add_content("price", ".price");
        g.add_intent_route("find_price", vec!["price".into()], None);
        store.insert(g);
    }

    let mut cache = IntentCache::new();
    let mut grammar_hits = 0;
    let mut dom_hits = 0;

    for i in 0..1000 {
        let domain = format!("site-{}.com", i % 200); // 50% have grammars
        let req = PerceptionRequest::extract_data(
            &format!("https://{domain}/page"),
            vec![DataField { name: "price".into(), field_type: FieldType::Currency }],
        );
        let decision = PerceptionRouter::route(&req, &store, &mut cache);
        match decision.primary_layer {
            PerceptionLayer::GrammarLookup => grammar_hits += 1,
            PerceptionLayer::DomExtraction => dom_hits += 1,
            _ => {}
        }
    }

    // 100 out of 200 unique domains have grammars (50%)
    // With 1000 requests mod 200, each domain hit 5 times
    assert_eq!(grammar_hits, 500);
    assert_eq!(dom_hits, 500);
}

#[test]
fn test_stress_significance_scoring_1000_grammars() {
    let scorer = SignificanceScorer::new(1000);
    let mut tiers = [0u32; 4]; // Archive, Cold, Standard, Active

    for i in 0..1000 {
        let mut g = SiteGrammar::new(format!("site-{i}.com"));
        g.query_success_count = i as u64;
        g.query_failure_count = (1000 - i) as u64 / 10;
        for j in 0..(i % 15) {
            g.add_content(format!("f{j}"), format!(".s{j}"));
        }

        let importance = (i as f32) / 1000.0;
        let score = scorer.score(&g, importance);

        assert!(score.score >= 0.0 && score.score <= 1.0);
        match score.tier {
            RetentionTier::Archive => tiers[0] += 1,
            RetentionTier::Cold => tiers[1] += 1,
            RetentionTier::Standard => tiers[2] += 1,
            RetentionTier::Active => tiers[3] += 1,
        }
    }

    // Should have a distribution across tiers
    assert!(tiers[0] > 0, "Should have some Archive-tier grammars");
    assert!(tiers[3] > 0, "Should have some Active-tier grammars");
}

#[test]
fn test_stress_v2_file_write_read_100_times() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("stress.avis");

    for iteration in 0..100 {
        let mut v2 = AvisStoreV2::new(512);
        v2.store.add(make_test_obs(0));
        let mut g = SiteGrammar::new(format!("iter-{iteration}.com"));
        g.add_content("f", ".s");
        v2.grammar_store.insert(g);

        AvisWriter::write_v2_to_file(&v2, &path).unwrap();
        let loaded = AvisReader::read_v2_from_file(&path).unwrap();
        assert_eq!(loaded.store.count(), 1);
        assert!(loaded.grammar_store.has(&format!("iter-{iteration}.com")));
    }
}
