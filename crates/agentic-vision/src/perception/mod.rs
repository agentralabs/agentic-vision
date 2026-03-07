//! Adaptive Perception Stack — the core of the Perception Revolution.
//!
//! Five-layer perception architecture where the agent starts at the cheapest
//! layer and only escalates when the task genuinely requires it.
//!
//! Layer 0: Semantic DOM Extraction (zero vision tokens)
//! Layer 1: Site Grammar (amortized to near-zero)
//! Layer 2: Intent-Scoped Extraction (proportional cost)
//! Layer 3: Delta Vision (only perceive what changed)
//! Layer 4: Scoped Screenshot (visual content only — last resort)

pub mod budget;
pub mod cache;
pub mod dom;
pub mod drift;
pub mod grammar;
pub mod router;
pub mod significance;
pub mod types;

pub use budget::{TokenBudget, TokenBudgetTier};
pub use cache::{IntentCache, IntentCacheEntry, IntentCacheKey};
pub use dom::{AccessibilityNode, AccessibilityRole, DomSnapshot};
pub use drift::{DriftDetector, DriftEvent, DriftSeverity};
pub use grammar::{
    ContentMapEntry, GrammarStatus, GrammarStore, IntentRoute, InteractionPattern,
    NavigationGrammar, NavigationType, SelectorType, SiteGrammar, StateIndicator,
};
pub use router::{PerceptionLayer, PerceptionResult, PerceptionRouter};
pub use significance::{SignificanceScore, SignificanceScorer};
pub use types::{FallbackStrategy, PerceptionIntent, PerceptionRequest};
