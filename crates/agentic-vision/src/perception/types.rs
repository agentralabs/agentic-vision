//! Core perception types: intent declaration, requests, and fallback strategies.

use serde::{Deserialize, Serialize};

use super::budget::TokenBudget;

/// The intent of a perception request. Determines which layer handles it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PerceptionIntent {
    /// Extract specific structured data fields (Layer 0 — zero vision tokens).
    ExtractData { fields: Vec<DataField> },

    /// Find an interactable element for a given action (Layer 0/1).
    FindInteractable { action: ActionType },

    /// Verify that the page is in an expected state (Layer 0/1).
    VerifyState { expected: String },

    /// Read content within a defined scope (Layer 2).
    ReadContent { scope: ContentScope },

    /// Monitor a page for changes against a baseline (Layer 3 — delta vision).
    MonitorChanges { baseline_hash: Option<String> },

    /// Analyze visual content (charts, diagrams) — requires screenshot (Layer 4).
    AnalyzeVisual,

    /// Read a scanned/image-based document (Layer 4).
    ReadDocument,

    /// Verify a CAPTCHA (Layer 4).
    VerifyCaptcha,

    /// Capture the full visual state — explicit last resort (Layer 4).
    CaptureFullVisual,
}

/// A data field to extract from a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataField {
    /// Semantic name of the field (e.g., "product_price", "page_title").
    pub name: String,

    /// Expected data type for validation.
    #[serde(default)]
    pub field_type: FieldType,
}

/// Expected type of an extracted field.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    #[default]
    Text,
    Number,
    Url,
    Boolean,
    DateTime,
    Currency,
}

/// Type of interaction action.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Click,
    TypeText,
    Select,
    Submit,
    Scroll,
    Navigate,
}

/// Scope of content to read.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentScope {
    /// Read the main content area only.
    MainContent,
    /// Read a specific section by selector or role.
    Section { selector: String },
    /// Read all text content.
    FullText,
    /// Read a list of items (e.g., search results).
    ItemList { container_selector: Option<String> },
}

/// Strategy when a perception layer fails.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackStrategy {
    /// Escalate to the next more expensive layer.
    #[default]
    Escalate,
    /// Return an error immediately.
    Fail,
    /// Return partial results from current layer.
    Partial,
    /// Skip to a specific layer.
    SkipTo { layer: u8 },
}

/// A complete perception request with intent, budget, and context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionRequest {
    /// What does the agent want to perceive?
    pub intent: PerceptionIntent,

    /// URL of the page (for grammar lookup).
    pub url: Option<String>,

    /// Domain extracted from URL (for grammar lookup).
    pub domain: Option<String>,

    /// Token budget for this request.
    #[serde(default)]
    pub budget: TokenBudget,

    /// What to do if the selected layer fails.
    #[serde(default)]
    pub fallback: FallbackStrategy,
}

impl PerceptionRequest {
    /// Create a data extraction request.
    pub fn extract_data(url: &str, fields: Vec<DataField>) -> Self {
        Self {
            intent: PerceptionIntent::ExtractData { fields },
            url: Some(url.to_string()),
            domain: extract_domain(url),
            budget: TokenBudget::surgical(),
            fallback: FallbackStrategy::Escalate,
        }
    }

    /// Create an interactable-finding request.
    pub fn find_interactable(url: &str, action: ActionType) -> Self {
        Self {
            intent: PerceptionIntent::FindInteractable { action },
            url: Some(url.to_string()),
            domain: extract_domain(url),
            budget: TokenBudget::focused(),
            fallback: FallbackStrategy::Escalate,
        }
    }

    /// Create a monitoring request.
    pub fn monitor(url: &str, baseline_hash: Option<String>) -> Self {
        Self {
            intent: PerceptionIntent::MonitorChanges { baseline_hash },
            url: Some(url.to_string()),
            domain: extract_domain(url),
            budget: TokenBudget::focused(),
            fallback: FallbackStrategy::Escalate,
        }
    }
}

/// Extract domain from a URL.
fn extract_domain(url: &str) -> Option<String> {
    let url = url.trim();
    // Strip protocol
    let after_proto = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };
    // Strip path
    let domain = if let Some(pos) = after_proto.find('/') {
        &after_proto[..pos]
    } else {
        after_proto
    };
    // Strip port
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://www.amazon.com/dp/B09G9HD6PD"),
            Some("www.amazon.com".into())
        );
        assert_eq!(
            extract_domain("http://localhost:8080/test"),
            Some("localhost".into())
        );
        assert_eq!(
            extract_domain("github.com/org/repo"),
            Some("github.com".into())
        );
        assert_eq!(extract_domain(""), None);
    }

    #[test]
    fn test_perception_request_extract_data() {
        let req = PerceptionRequest::extract_data(
            "https://amazon.com/dp/X",
            vec![DataField {
                name: "price".into(),
                field_type: FieldType::Currency,
            }],
        );
        assert_eq!(req.domain, Some("amazon.com".into()));
        assert!(matches!(req.intent, PerceptionIntent::ExtractData { .. }));
    }
}
