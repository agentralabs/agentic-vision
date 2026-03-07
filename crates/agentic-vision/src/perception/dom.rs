//! DOM and Accessibility Tree types for Layer 0 extraction.
//!
//! These types represent the structured data extracted from a browser's
//! accessibility tree and DOM, enabling zero-vision-token page queries.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// ARIA role of an accessibility node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccessibilityRole {
    Button,
    Link,
    Textbox,
    Heading,
    Image,
    List,
    ListItem,
    Navigation,
    Main,
    Complementary,
    Banner,
    ContentInfo,
    Form,
    Search,
    Table,
    Row,
    Cell,
    Alert,
    Dialog,
    Menu,
    MenuItem,
    Tab,
    TabPanel,
    Tree,
    TreeItem,
    Checkbox,
    Radio,
    Slider,
    Spinbutton,
    Combobox,
    Grid,
    Region,
    Article,
    Group,
    Separator,
    /// Catch-all for unlisted roles.
    Other(String),
}

/// A single node in the accessibility tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityNode {
    /// Node ID (unique within the tree).
    pub node_id: u64,

    /// ARIA role.
    pub role: AccessibilityRole,

    /// Accessible name (visible text or aria-label).
    pub name: Option<String>,

    /// Accessible description.
    pub description: Option<String>,

    /// Accessible value (for inputs, sliders, etc.).
    pub value: Option<String>,

    /// CSS selector path to this element.
    pub selector: Option<String>,

    /// Whether this element is interactive (clickable, editable).
    #[serde(default)]
    pub interactive: bool,

    /// Whether this element is currently visible.
    #[serde(default = "default_true")]
    pub visible: bool,

    /// Bounding box in viewport coordinates (if available).
    pub bounds: Option<NodeBounds>,

    /// HTML attributes that may be useful for grammar learning.
    #[serde(default)]
    pub attributes: HashMap<String, String>,

    /// Child node IDs.
    #[serde(default)]
    pub children: Vec<u64>,
}

fn default_true() -> bool {
    true
}

/// Bounding box of a DOM element.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NodeBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// A complete snapshot of a page's accessibility tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomSnapshot {
    /// URL of the page.
    pub url: String,

    /// Domain extracted from URL.
    pub domain: String,

    /// Structural hash (blake3 of node structure) for drift detection.
    pub structural_hash: String,

    /// Content hash (blake3 of text content) for change detection.
    pub content_hash: String,

    /// When this snapshot was taken.
    pub captured_at: u64,

    /// Root nodes of the accessibility tree.
    pub root_nodes: Vec<u64>,

    /// All nodes indexed by ID.
    pub nodes: HashMap<u64, AccessibilityNode>,

    /// Page title.
    pub title: Option<String>,

    /// Estimated text token count for the full page.
    pub estimated_tokens: u32,
}

impl DomSnapshot {
    /// Create a new empty snapshot.
    pub fn new(url: impl Into<String>, domain: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            url: url.into(),
            domain: domain.into(),
            structural_hash: String::new(),
            content_hash: String::new(),
            captured_at: now,
            root_nodes: Vec::new(),
            nodes: HashMap::new(),
            title: None,
            estimated_tokens: 0,
        }
    }

    /// Add a node to the snapshot.
    pub fn add_node(&mut self, node: AccessibilityNode) {
        self.nodes.insert(node.node_id, node);
    }

    /// Find nodes by role.
    pub fn find_by_role(&self, role: &AccessibilityRole) -> Vec<&AccessibilityNode> {
        self.nodes.values().filter(|n| &n.role == role).collect()
    }

    /// Find nodes matching a CSS class pattern in their selector.
    pub fn find_by_selector_pattern(&self, pattern: &str) -> Vec<&AccessibilityNode> {
        self.nodes
            .values()
            .filter(|n| {
                n.selector
                    .as_ref()
                    .is_some_and(|s| s.contains(pattern))
            })
            .collect()
    }

    /// Find interactive elements.
    pub fn interactive_elements(&self) -> Vec<&AccessibilityNode> {
        self.nodes.values().filter(|n| n.interactive).collect()
    }

    /// Extract text content from the tree (for content hashing).
    pub fn text_content(&self) -> String {
        let mut texts = Vec::new();
        for node in self.nodes.values() {
            if let Some(ref name) = node.name {
                if !name.is_empty() {
                    texts.push(name.as_str());
                }
            }
            if let Some(ref value) = node.value {
                if !value.is_empty() {
                    texts.push(value.as_str());
                }
            }
        }
        texts.join(" ")
    }

    /// Compute and set the structural hash (based on node roles and hierarchy).
    pub fn compute_structural_hash(&mut self) {
        let mut hasher = blake3::Hasher::new();
        let mut sorted_ids: Vec<_> = self.nodes.keys().copied().collect();
        sorted_ids.sort();
        for id in sorted_ids {
            if let Some(node) = self.nodes.get(&id) {
                hasher.update(format!("{:?}", node.role).as_bytes());
                if let Some(ref sel) = node.selector {
                    hasher.update(sel.as_bytes());
                }
                for child_id in &node.children {
                    hasher.update(&child_id.to_le_bytes());
                }
            }
        }
        self.structural_hash = format!("blake3:{}", hasher.finalize().to_hex());
    }

    /// Compute and set the content hash (based on text content).
    pub fn compute_content_hash(&mut self) {
        let content = self.text_content();
        let hash = blake3::hash(content.as_bytes());
        self.content_hash = format!("blake3:{}", hash.to_hex());
    }
}

/// Result of querying a DOM snapshot with a selector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomQueryResult {
    /// The extracted value(s).
    pub values: Vec<String>,
    /// Which selector matched.
    pub matched_selector: String,
    /// Number of nodes matched.
    pub match_count: usize,
    /// Estimated tokens this result costs.
    pub token_cost: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_snapshot_creation() {
        let snap = DomSnapshot::new("https://example.com", "example.com");
        assert_eq!(snap.url, "https://example.com");
        assert_eq!(snap.domain, "example.com");
        assert!(snap.nodes.is_empty());
    }

    #[test]
    fn test_add_and_find_nodes() {
        let mut snap = DomSnapshot::new("https://example.com", "example.com");
        snap.add_node(AccessibilityNode {
            node_id: 1,
            role: AccessibilityRole::Button,
            name: Some("Submit".into()),
            description: None,
            value: None,
            selector: Some("button#submit".into()),
            interactive: true,
            visible: true,
            bounds: None,
            attributes: HashMap::new(),
            children: vec![],
        });
        snap.add_node(AccessibilityNode {
            node_id: 2,
            role: AccessibilityRole::Heading,
            name: Some("Product Title".into()),
            description: None,
            value: None,
            selector: Some("h1#title".into()),
            interactive: false,
            visible: true,
            bounds: None,
            attributes: HashMap::new(),
            children: vec![],
        });

        assert_eq!(snap.find_by_role(&AccessibilityRole::Button).len(), 1);
        assert_eq!(snap.interactive_elements().len(), 1);
        assert_eq!(snap.find_by_selector_pattern("button").len(), 1);
    }

    #[test]
    fn test_structural_hash() {
        let mut snap = DomSnapshot::new("https://example.com", "example.com");
        snap.add_node(AccessibilityNode {
            node_id: 1,
            role: AccessibilityRole::Main,
            name: None,
            description: None,
            value: None,
            selector: Some("main".into()),
            interactive: false,
            visible: true,
            bounds: None,
            attributes: HashMap::new(),
            children: vec![],
        });
        snap.compute_structural_hash();
        assert!(snap.structural_hash.starts_with("blake3:"));
        assert!(snap.structural_hash.len() > 10);
    }
}
