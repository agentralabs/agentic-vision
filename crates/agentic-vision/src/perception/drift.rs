//! Grammar Drift Detection — automatically detect when sites change.
//!
//! On every visit, AgenticVision computes a structural hash of key grammar nodes
//! and compares against the stored hash. Mismatches trigger partial re-learning.

use serde::{Deserialize, Serialize};

/// Severity of a grammar drift event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftSeverity {
    /// Minor: a few selectors changed but most grammar still works.
    Minor,
    /// Moderate: significant sections need re-learning.
    Moderate,
    /// Major: site appears to have been redesigned.
    Major,
}

/// A recorded drift event for a site grammar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftEvent {
    /// Domain affected.
    pub domain: String,

    /// Previous structural hash.
    pub old_hash: String,

    /// New structural hash.
    pub new_hash: String,

    /// Severity classification.
    pub severity: DriftSeverity,

    /// Content map keys that are now broken.
    pub broken_selectors: Vec<String>,

    /// Content map keys that still work.
    pub working_selectors: Vec<String>,

    /// When this drift was detected.
    pub detected_at: u64,

    /// Whether partial re-learning has been triggered.
    pub relearn_triggered: bool,

    /// Grammar version before drift.
    pub old_version: String,

    /// Grammar version after re-learning (filled after relearn).
    pub new_version: Option<String>,
}

/// Drift detection engine.
pub struct DriftDetector;

impl DriftDetector {
    /// Compare a stored grammar's structural hash against a new snapshot's hash.
    /// Returns a DriftEvent if drift is detected, None if hashes match.
    pub fn detect(
        domain: &str,
        stored_hash: &str,
        new_hash: &str,
        grammar_version: &str,
        broken_keys: Vec<String>,
        working_keys: Vec<String>,
    ) -> Option<DriftEvent> {
        if stored_hash == new_hash {
            return None;
        }

        let total = broken_keys.len() + working_keys.len();
        let broken_ratio = if total == 0 {
            0.0
        } else {
            broken_keys.len() as f32 / total as f32
        };

        let severity = if broken_ratio > 0.5 {
            DriftSeverity::Major
        } else if broken_ratio > 0.2 {
            DriftSeverity::Moderate
        } else {
            DriftSeverity::Minor
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Some(DriftEvent {
            domain: domain.to_string(),
            old_hash: stored_hash.to_string(),
            new_hash: new_hash.to_string(),
            severity,
            broken_selectors: broken_keys,
            working_selectors: working_keys,
            detected_at: now,
            relearn_triggered: false,
            old_version: grammar_version.to_string(),
            new_version: None,
        })
    }

    /// Estimate the re-learning cost in tokens based on severity.
    pub fn estimated_relearn_cost(severity: DriftSeverity) -> u32 {
        match severity {
            DriftSeverity::Minor => 200,
            DriftSeverity::Moderate => 500,
            DriftSeverity::Major => 1500,
        }
    }
}

/// History of drift events for a domain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DriftHistory {
    pub events: Vec<DriftEvent>,
}

impl DriftHistory {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Record a new drift event.
    pub fn record(&mut self, event: DriftEvent) {
        self.events.push(event);
    }

    /// Get the most recent drift event for a domain.
    pub fn latest(&self, domain: &str) -> Option<&DriftEvent> {
        self.events
            .iter()
            .rev()
            .find(|e| e.domain == domain)
    }

    /// Count drift events for a domain.
    pub fn count_for_domain(&self, domain: &str) -> usize {
        self.events.iter().filter(|e| e.domain == domain).count()
    }

    /// Estimate drift frequency for a domain (events per year).
    pub fn drift_frequency(&self, domain: &str) -> f32 {
        let domain_events: Vec<_> = self.events.iter().filter(|e| e.domain == domain).collect();
        if domain_events.len() < 2 {
            return 0.0;
        }
        let first = domain_events.first().unwrap().detected_at;
        let last = domain_events.last().unwrap().detected_at;
        let span_secs = last.saturating_sub(first);
        if span_secs == 0 {
            return 0.0;
        }
        let span_years = span_secs as f32 / (365.25 * 24.0 * 3600.0);
        if span_years < 0.01 {
            return 0.0;
        }
        domain_events.len() as f32 / span_years
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_drift_on_matching_hash() {
        let result = DriftDetector::detect("test.com", "abc123", "abc123", "v1", vec![], vec![]);
        assert!(result.is_none());
    }

    #[test]
    fn test_minor_drift() {
        let result = DriftDetector::detect(
            "test.com",
            "abc",
            "def",
            "v1",
            vec!["price".into()],
            vec!["title".into(), "nav".into(), "footer".into(), "search".into()],
        );
        let event = result.unwrap();
        assert_eq!(event.severity, DriftSeverity::Minor);
        assert_eq!(event.domain, "test.com");
    }

    #[test]
    fn test_major_drift() {
        let result = DriftDetector::detect(
            "test.com",
            "abc",
            "def",
            "v1",
            vec!["price".into(), "title".into(), "nav".into()],
            vec!["footer".into()],
        );
        let event = result.unwrap();
        assert_eq!(event.severity, DriftSeverity::Major);
    }

    #[test]
    fn test_relearn_cost() {
        assert_eq!(DriftDetector::estimated_relearn_cost(DriftSeverity::Minor), 200);
        assert_eq!(DriftDetector::estimated_relearn_cost(DriftSeverity::Major), 1500);
    }

    #[test]
    fn test_drift_history() {
        let mut history = DriftHistory::new();
        let event = DriftEvent {
            domain: "test.com".into(),
            old_hash: "a".into(),
            new_hash: "b".into(),
            severity: DriftSeverity::Minor,
            broken_selectors: vec![],
            working_selectors: vec![],
            detected_at: 1000,
            relearn_triggered: false,
            old_version: "v1".into(),
            new_version: None,
        };
        history.record(event);
        assert_eq!(history.count_for_domain("test.com"), 1);
        assert!(history.latest("test.com").is_some());
        assert!(history.latest("other.com").is_none());
    }
}
