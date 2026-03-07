//! Grammar Significance Scoring — determines retention priority.
//!
//! Factors: usage frequency (0.35), success rate (0.25), site importance (0.20),
//! uniqueness (0.10), recency (0.10).

use serde::{Deserialize, Serialize};

use super::grammar::SiteGrammar;

/// Weight factors for significance scoring.
const WEIGHT_USAGE: f32 = 0.35;
const WEIGHT_SUCCESS: f32 = 0.25;
const WEIGHT_IMPORTANCE: f32 = 0.20;
const WEIGHT_UNIQUENESS: f32 = 0.10;
const WEIGHT_RECENCY: f32 = 0.10;

/// Retention tier based on significance score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionTier {
    /// Score > 0.7: active tier, replicated to community pool.
    Active,
    /// 0.4 - 0.7: standard tier, kept in .avis hot path.
    Standard,
    /// 0.2 - 0.4: cold tier, moved to .vision.db SQLite.
    Cold,
    /// < 0.2: candidate for archival.
    Archive,
}

/// Significance score with breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificanceScore {
    /// Overall score [0.0, 1.0].
    pub score: f32,
    /// Usage frequency component.
    pub usage_component: f32,
    /// Success rate component.
    pub success_component: f32,
    /// Site importance component.
    pub importance_component: f32,
    /// Uniqueness component.
    pub uniqueness_component: f32,
    /// Recency component.
    pub recency_component: f32,
    /// Retention tier.
    pub tier: RetentionTier,
}

impl SignificanceScore {
    /// Determine retention tier from score.
    pub fn tier_from_score(score: f32) -> RetentionTier {
        if score > 0.7 {
            RetentionTier::Active
        } else if score > 0.4 {
            RetentionTier::Standard
        } else if score > 0.2 {
            RetentionTier::Cold
        } else {
            RetentionTier::Archive
        }
    }
}

/// Scorer for grammar significance.
pub struct SignificanceScorer {
    /// Maximum usage count across all grammars (for normalization).
    max_usage: u64,
}

impl SignificanceScorer {
    pub fn new(max_usage: u64) -> Self {
        Self {
            max_usage: max_usage.max(1),
        }
    }

    /// Score a grammar's significance.
    pub fn score(&self, grammar: &SiteGrammar, site_importance: f32) -> SignificanceScore {
        let usage = self.usage_score(grammar);
        let success = grammar.success_rate();
        let importance = site_importance.clamp(0.0, 1.0);
        let uniqueness = self.uniqueness_score(grammar);
        let recency = self.recency_score(grammar);

        let score = usage * WEIGHT_USAGE
            + success * WEIGHT_SUCCESS
            + importance * WEIGHT_IMPORTANCE
            + uniqueness * WEIGHT_UNIQUENESS
            + recency * WEIGHT_RECENCY;

        let score = score.clamp(0.0, 1.0);

        SignificanceScore {
            score,
            usage_component: usage,
            success_component: success,
            importance_component: importance,
            uniqueness_component: uniqueness,
            recency_component: recency,
            tier: SignificanceScore::tier_from_score(score),
        }
    }

    /// Usage frequency normalized against max usage.
    fn usage_score(&self, grammar: &SiteGrammar) -> f32 {
        let total = grammar.query_success_count + grammar.query_failure_count;
        (total as f32 / self.max_usage as f32).min(1.0)
    }

    /// Uniqueness: how many content map entries are non-generic.
    /// Simple heuristic: more entries = more specialized = more unique.
    fn uniqueness_score(&self, grammar: &SiteGrammar) -> f32 {
        let entries = grammar.content_map.len();
        // 10+ entries is considered very specialized
        (entries as f32 / 10.0).min(1.0)
    }

    /// Recency: how recently was this grammar verified?
    fn recency_score(&self, grammar: &SiteGrammar) -> f32 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let last = grammar.last_verified.unwrap_or(grammar.updated_at);
        let age_secs = now.saturating_sub(last);
        let age_days = age_secs as f32 / 86400.0;

        // Verified today: 1.0, verified 30 days ago: 0.5, 365 days ago: ~0.0
        (1.0 - (age_days / 365.0)).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_from_score() {
        assert_eq!(
            SignificanceScore::tier_from_score(0.8),
            RetentionTier::Active
        );
        assert_eq!(
            SignificanceScore::tier_from_score(0.5),
            RetentionTier::Standard
        );
        assert_eq!(SignificanceScore::tier_from_score(0.3), RetentionTier::Cold);
        assert_eq!(
            SignificanceScore::tier_from_score(0.1),
            RetentionTier::Archive
        );
    }

    #[test]
    fn test_significance_scoring() {
        let mut grammar = SiteGrammar::new("amazon.com");
        grammar.query_success_count = 100;
        grammar.query_failure_count = 5;
        grammar.add_content("price", ".a-price");
        grammar.add_content("title", "#title");
        grammar.add_content("rating", "#rating");

        let scorer = SignificanceScorer::new(200);
        let score = scorer.score(&grammar, 0.9);

        assert!(score.score > 0.0);
        assert!(score.score <= 1.0);
        assert!(score.usage_component > 0.0);
        assert!(score.success_component > 0.9);
    }

    #[test]
    fn test_zero_usage_grammar() {
        let grammar = SiteGrammar::new("unused.com");
        let scorer = SignificanceScorer::new(100);
        let score = scorer.score(&grammar, 0.0);

        // Low score for unused grammar
        assert!(score.score < 0.3);
        assert_eq!(score.usage_component, 0.0);
    }
}
