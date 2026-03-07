//! Token budget enforcement — hard caps on perception cost per request.

use serde::{Deserialize, Serialize};

/// Budget tier determining the maximum token cost for a perception request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenBudgetTier {
    /// 0-50 tokens: single-field data extraction.
    Surgical,
    /// 50-300 tokens: multi-field extraction or simple interaction.
    Focused,
    /// 300-800 tokens: page section analysis or complex navigation.
    Contextual,
    /// 800-2000 tokens: genuinely visual content (charts, diagrams).
    Visual,
    /// 2000+ tokens: full page screenshot — last resort, must be explicit.
    FullPage,
}

impl TokenBudgetTier {
    /// Maximum tokens allowed for this tier.
    pub fn max_tokens(self) -> u32 {
        match self {
            Self::Surgical => 50,
            Self::Focused => 300,
            Self::Contextual => 800,
            Self::Visual => 2000,
            Self::FullPage => 5000,
        }
    }
}

/// Token budget for a perception request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    /// The budget tier.
    pub tier: TokenBudgetTier,

    /// Hard cap on tokens (overrides tier default if set).
    pub max_tokens: Option<u32>,

    /// Track tokens used so far in this request.
    #[serde(default)]
    pub tokens_used: u32,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            tier: TokenBudgetTier::Focused,
            max_tokens: None,
            tokens_used: 0,
        }
    }
}

impl TokenBudget {
    /// Create a Surgical budget (0-50 tokens).
    pub fn surgical() -> Self {
        Self {
            tier: TokenBudgetTier::Surgical,
            max_tokens: None,
            tokens_used: 0,
        }
    }

    /// Create a Focused budget (50-300 tokens).
    pub fn focused() -> Self {
        Self {
            tier: TokenBudgetTier::Focused,
            max_tokens: None,
            tokens_used: 0,
        }
    }

    /// Create a Contextual budget (300-800 tokens).
    pub fn contextual() -> Self {
        Self {
            tier: TokenBudgetTier::Contextual,
            max_tokens: None,
            tokens_used: 0,
        }
    }

    /// Create a Visual budget (800-2000 tokens).
    pub fn visual() -> Self {
        Self {
            tier: TokenBudgetTier::Visual,
            max_tokens: None,
            tokens_used: 0,
        }
    }

    /// Create a FullPage budget (2000+ tokens — last resort).
    pub fn full_page() -> Self {
        Self {
            tier: TokenBudgetTier::FullPage,
            max_tokens: None,
            tokens_used: 0,
        }
    }

    /// The effective maximum tokens for this budget.
    pub fn effective_max(&self) -> u32 {
        self.max_tokens.unwrap_or_else(|| self.tier.max_tokens())
    }

    /// Remaining tokens in this budget.
    pub fn remaining(&self) -> u32 {
        self.effective_max().saturating_sub(self.tokens_used)
    }

    /// Whether the budget has been exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.tokens_used >= self.effective_max()
    }

    /// Record token usage. Returns true if within budget.
    pub fn consume(&mut self, tokens: u32) -> bool {
        self.tokens_used += tokens;
        self.tokens_used <= self.effective_max()
    }
}

/// Record of tokens spent on a perception call (for auditing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudgetRecord {
    /// Which perception layer handled this.
    pub layer: u8,
    /// Tokens actually used.
    pub tokens_used: u32,
    /// Budget tier that was set.
    pub tier: TokenBudgetTier,
    /// URL involved.
    pub url: Option<String>,
    /// Intent type name.
    pub intent_type: String,
    /// Timestamp.
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_tiers() {
        assert_eq!(TokenBudgetTier::Surgical.max_tokens(), 50);
        assert_eq!(TokenBudgetTier::FullPage.max_tokens(), 5000);
    }

    #[test]
    fn test_budget_consumption() {
        let mut budget = TokenBudget::surgical();
        assert_eq!(budget.remaining(), 50);

        assert!(budget.consume(30));
        assert_eq!(budget.remaining(), 20);

        assert!(!budget.consume(30)); // over budget
        assert!(budget.is_exhausted());
    }

    #[test]
    fn test_custom_max() {
        let budget = TokenBudget {
            tier: TokenBudgetTier::Surgical,
            max_tokens: Some(100),
            tokens_used: 0,
        };
        assert_eq!(budget.effective_max(), 100);
    }
}
