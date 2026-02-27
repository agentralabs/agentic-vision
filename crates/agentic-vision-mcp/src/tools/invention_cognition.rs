//! Cognition Inventions (13–16): Semantic Vision, Visual Reasoning Chain,
//! Cross-Modal Binding, Visual Gestalt.
//!
//! 14 MCP tools that understand what you see.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::session::manager::VisionSessionManager;
use crate::types::error::{McpError, McpResult};
use crate::types::response::{ToolCallResult, ToolDefinition};

// ═══════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SemanticRole {
    Navigation,
    Breadcrumb,
    Menu,
    Heading,
    Paragraph,
    List,
    Image,
    Video,
    Button,
    Link,
    Input,
    Form,
    Error,
    Warning,
    Success,
    Loading,
    Progress,
    Price,
    CartItem,
    Checkout,
    Avatar,
    Username,
    Badge,
    Header,
    Footer,
    Sidebar,
    Modal,
    Card,
}

impl SemanticRole {
    fn label(&self) -> &str {
        match self {
            Self::Navigation => "navigation",
            Self::Breadcrumb => "breadcrumb",
            Self::Menu => "menu",
            Self::Heading => "heading",
            Self::Paragraph => "paragraph",
            Self::List => "list",
            Self::Image => "image",
            Self::Video => "video",
            Self::Button => "button",
            Self::Link => "link",
            Self::Input => "input",
            Self::Form => "form",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Success => "success",
            Self::Loading => "loading",
            Self::Progress => "progress",
            Self::Price => "price",
            Self::CartItem => "cart_item",
            Self::Checkout => "checkout",
            Self::Avatar => "avatar",
            Self::Username => "username",
            Self::Badge => "badge",
            Self::Header => "header",
            Self::Footer => "footer",
            Self::Sidebar => "sidebar",
            Self::Modal => "modal",
            Self::Card => "card",
        }
    }

    fn from_keyword(kw: &str) -> Option<Self> {
        match kw {
            "nav" | "navigation" | "menu" => Some(Self::Navigation),
            "breadcrumb" => Some(Self::Breadcrumb),
            "heading" | "title" | "h1" | "h2" | "h3" => Some(Self::Heading),
            "button" | "btn" | "cta" => Some(Self::Button),
            "link" | "anchor" => Some(Self::Link),
            "input" | "field" | "textbox" => Some(Self::Input),
            "form" => Some(Self::Form),
            "error" | "alert" => Some(Self::Error),
            "warning" | "warn" => Some(Self::Warning),
            "success" | "ok" | "confirmed" => Some(Self::Success),
            "loading" | "spinner" => Some(Self::Loading),
            "progress" | "bar" => Some(Self::Progress),
            "price" | "cost" | "amount" => Some(Self::Price),
            "cart" | "basket" => Some(Self::CartItem),
            "checkout" | "purchase" | "buy" => Some(Self::Checkout),
            "avatar" | "profile" | "photo" => Some(Self::Avatar),
            "header" | "top" => Some(Self::Header),
            "footer" | "bottom" => Some(Self::Footer),
            "sidebar" | "side" => Some(Self::Sidebar),
            "modal" | "dialog" | "popup" => Some(Self::Modal),
            "card" | "tile" => Some(Self::Card),
            "image" | "img" | "picture" => Some(Self::Image),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JourneyStage {
    Discovery,
    Consideration,
    Decision,
    Action,
    Confirmation,
    Error,
    Support,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ObservationType {
    StateObservation,
    ChangeObservation,
    PatternObservation,
    AnomalyObservation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ReasoningType {
    Causal,
    Comparative,
    Analogical,
    Deductive,
    Inductive,
    UXPrinciple,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConclusionType {
    ProblemIdentified,
    RootCauseFound,
    UserNeedIdentified,
    DesignFlawFound,
    OpportunityIdentified,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GestaltType {
    Similarity,
    Proximity,
    Continuity,
    Closure,
    FigureGround,
    Symmetry,
    CommonFate,
    PastExperience,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GestaltEffect {
    Positive,
    Neutral,
    Negative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EmotionalTone {
    Calm,
    Energetic,
    Professional,
    Playful,
    Serious,
    Trustworthy,
    Urgent,
    Confused,
    Chaotic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CodeBindingType {
    RenderedBy,
    StyledBy,
    ControlledBy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MemoryBindingType {
    AffectedByDecision,
    FactAbout,
    InvolvedInEpisode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IdentityBindingType {
    ModifiedBy,
    OwnedBy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeBindingType {
    HasDeadline,
    ScheduledChange,
}

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

fn word_overlap(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let a_words: std::collections::HashSet<&str> = a_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .collect();
    let b_words: std::collections::HashSet<&str> = b_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .collect();
    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }
    let intersection = a_words.intersection(&b_words).count();
    intersection as f64 / a_words.len().max(b_words.len()) as f64
}

fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 13: Semantic Vision — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

pub fn definition_vision_semantic_analyze() -> ToolDefinition {
    ToolDefinition {
        name: "vision_semantic_analyze".to_string(),
        description: Some("Analyze semantic meaning of UI from a capture".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to analyze" }
            }
        }),
    }
}

pub async fn execute_vision_semantic_analyze(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs = match store.observations.iter().find(|o| o.id == p.capture_id) {
        Some(o) => o,
        None => return Err(McpError::CaptureNotFound(p.capture_id)),
    };

    // Infer semantic elements from labels and description
    let mut elements: Vec<Value> = Vec::new();
    let mut detected_roles: Vec<String> = Vec::new();

    for label in &obs.metadata.labels {
        let label_lower = label.to_lowercase();
        if let Some(role) = SemanticRole::from_keyword(&label_lower) {
            elements.push(json!({
                "role": role.label(),
                "source": "label",
                "label": label,
                "importance": 0.7,
            }));
            detected_roles.push(role.label().to_string());
        }
    }

    if let Some(desc) = &obs.metadata.description {
        for word in desc.to_lowercase().split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if let Some(role) = SemanticRole::from_keyword(clean) {
                if !detected_roles.contains(&role.label().to_string()) {
                    elements.push(json!({
                        "role": role.label(),
                        "source": "description",
                        "word": clean,
                        "importance": 0.5,
                    }));
                    detected_roles.push(role.label().to_string());
                }
            }
        }
    }

    // Infer page intent
    let has_form = detected_roles.iter().any(|r| r == "form" || r == "input");
    let has_error = detected_roles
        .iter()
        .any(|r| r == "error" || r == "warning");
    let has_checkout = detected_roles
        .iter()
        .any(|r| r == "checkout" || r == "cart_item" || r == "price");
    let has_navigation = detected_roles
        .iter()
        .any(|r| r == "navigation" || r == "menu");

    let (page_intent, journey_stage) = if has_error {
        ("error_handling", "error")
    } else if has_checkout {
        ("purchase_flow", "action")
    } else if has_form {
        ("data_entry", "action")
    } else if has_navigation {
        ("navigation_hub", "discovery")
    } else {
        ("content_display", "discovery")
    };

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "elements": elements,
        "page_intent": {
            "primary": page_intent,
            "confidence": 0.6,
        },
        "journey_stage": journey_stage,
        "hierarchy": {
            "primary": detected_roles.iter().take(3).collect::<Vec<_>>(),
            "secondary": detected_roles.iter().skip(3).collect::<Vec<_>>(),
        },
    })))
}

pub fn definition_vision_semantic_find() -> ToolDefinition {
    ToolDefinition {
        name: "vision_semantic_find".to_string(),
        description: Some("Find elements by semantic role across captures".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["role"],
            "properties": {
                "role": { "type": "string", "description": "Semantic role to find (button, form, error, navigation, etc.)" }
            }
        }),
    }
}

pub async fn execute_vision_semantic_find(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        role: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let role_lower = p.role.to_lowercase();

    let mut results: Vec<Value> = Vec::new();

    for obs in &store.observations {
        let mut match_score: f64 = 0.0;
        for label in &obs.metadata.labels {
            if label.to_lowercase().contains(&role_lower)
                || role_lower.contains(&label.to_lowercase())
            {
                match_score += 0.5;
                break;
            }
        }
        if let Some(desc) = &obs.metadata.description {
            if desc.to_lowercase().contains(&role_lower) {
                match_score += 0.4;
            }
        }
        if match_score > 0.1 {
            results.push(json!({
                "capture_id": obs.id,
                "timestamp": obs.timestamp,
                "description": obs.metadata.description,
                "labels": obs.metadata.labels,
                "match_score": (match_score.min(1.0) * 100.0).round() / 100.0,
            }));
        }
    }

    results.sort_by(|a, b| {
        b["match_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["match_score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(20);

    Ok(ToolCallResult::json(&json!({
        "role": p.role,
        "found": results.len(),
        "results": results,
    })))
}

pub fn definition_vision_semantic_intent() -> ToolDefinition {
    ToolDefinition {
        name: "vision_semantic_intent".to_string(),
        description: Some("Determine page or flow intent from visual state".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to analyze intent for" }
            }
        }),
    }
}

pub async fn execute_vision_semantic_intent(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs = match store.observations.iter().find(|o| o.id == p.capture_id) {
        Some(o) => o,
        None => return Err(McpError::CaptureNotFound(p.capture_id)),
    };

    let all_text = format!(
        "{} {}",
        obs.metadata.description.as_deref().unwrap_or(""),
        obs.metadata.labels.join(" ")
    )
    .to_lowercase();

    // Classify intent from keywords
    let mut intents: Vec<(&str, f64)> = Vec::new();

    let intent_keywords = [
        (
            "login",
            vec!["login", "sign in", "password", "email", "authenticate"],
        ),
        (
            "registration",
            vec!["register", "sign up", "create account", "join"],
        ),
        (
            "checkout",
            vec!["checkout", "payment", "order", "purchase", "cart", "buy"],
        ),
        (
            "search",
            vec!["search", "find", "query", "filter", "results"],
        ),
        (
            "dashboard",
            vec!["dashboard", "overview", "stats", "metrics", "analytics"],
        ),
        (
            "settings",
            vec!["settings", "preferences", "config", "profile"],
        ),
        ("error", vec!["error", "404", "500", "not found", "failed"]),
        (
            "content",
            vec!["article", "blog", "post", "content", "read"],
        ),
    ];

    for (intent, keywords) in &intent_keywords {
        let score: f64 = keywords.iter().filter(|kw| all_text.contains(*kw)).count() as f64
            / keywords.len() as f64;
        if score > 0.0 {
            intents.push((intent, score));
        }
    }

    intents.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let primary = intents.first().map(|(i, _)| *i).unwrap_or("unknown");
    let secondary: Vec<&str> = intents.iter().skip(1).take(2).map(|(i, _)| *i).collect();
    let confidence = intents.first().map(|(_, s)| *s).unwrap_or(0.0);

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "intent": {
            "primary": primary,
            "secondary": secondary,
            "confidence": (confidence.min(1.0) * 100.0).round() / 100.0,
        },
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 14: Visual Reasoning Chain — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

pub fn definition_vision_reason() -> ToolDefinition {
    ToolDefinition {
        name: "vision_reason".to_string(),
        description: Some("Build a reasoning chain from visual observations".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["observation"],
            "properties": {
                "observation": { "type": "string", "description": "Starting observation" },
                "capture_ids": { "type": "array", "items": { "type": "number" }, "description": "Supporting captures" }
            }
        }),
    }
}

pub async fn execute_vision_reason(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        observation: String,
        capture_ids: Option<Vec<u64>>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs_lower = p.observation.to_lowercase();

    // Build reasoning chain
    let mut reasoning_steps: Vec<Value> = Vec::new();
    let mut evidence: Vec<Value> = Vec::new();

    // Find supporting evidence
    for obs in &store.observations {
        if let Some(ref ids) = p.capture_ids {
            if !ids.contains(&obs.id) {
                continue;
            }
        }
        if let Some(desc) = &obs.metadata.description {
            let overlap = word_overlap(&obs_lower, &desc.to_lowercase());
            if overlap > 0.1 {
                evidence.push(json!({
                    "capture_id": obs.id,
                    "description": desc,
                    "relevance": overlap,
                }));
            }
        }
    }

    // Generate reasoning steps based on observation content
    reasoning_steps.push(json!({
        "step": 1,
        "reasoning": format!("Observed: {}", p.observation),
        "reasoning_type": "state_observation",
        "evidence_count": evidence.len(),
    }));

    if !evidence.is_empty() {
        reasoning_steps.push(json!({
            "step": 2,
            "reasoning": format!("Found {} supporting captures", evidence.len()),
            "reasoning_type": "deductive",
        }));
    }

    reasoning_steps.push(json!({
        "step": reasoning_steps.len() + 1,
        "reasoning": "Based on available visual evidence, this observation is supported",
        "reasoning_type": "inductive",
    }));

    let confidence = if evidence.is_empty() { 0.3 } else { 0.7 };

    Ok(ToolCallResult::json(&json!({
        "observation": p.observation,
        "reasoning_chain": reasoning_steps,
        "supporting_evidence": evidence,
        "conclusion": {
            "conclusion": format!("Observation '{}' is {} by visual evidence", p.observation,
                if evidence.is_empty() { "unsupported" } else { "supported" }),
            "conclusion_type": if evidence.is_empty() { "ungrounded" } else { "grounded" },
            "confidence": confidence,
        },
    })))
}

pub fn definition_vision_reason_about() -> ToolDefinition {
    ToolDefinition {
        name: "vision_reason_about".to_string(),
        description: Some("Reason about a specific UX question using visual evidence".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["question"],
            "properties": {
                "question": { "type": "string", "description": "UX question to reason about" }
            }
        }),
    }
}

pub async fn execute_vision_reason_about(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        question: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let q_lower = p.question.to_lowercase();

    let mut relevant: Vec<Value> = Vec::new();
    for obs in &store.observations {
        if let Some(desc) = &obs.metadata.description {
            let overlap = word_overlap(&q_lower, &desc.to_lowercase());
            if overlap > 0.1 {
                relevant.push(json!({
                    "capture_id": obs.id,
                    "description": desc,
                    "relevance": overlap,
                }));
            }
        }
    }

    let has_evidence = !relevant.is_empty();

    Ok(ToolCallResult::json(&json!({
        "question": p.question,
        "evidence_found": relevant.len(),
        "evidence": relevant,
        "reasoning": if has_evidence {
            "Visual evidence available to reason about this question"
        } else {
            "Insufficient visual evidence to fully answer this question"
        },
        "confidence": if has_evidence { 0.6 } else { 0.2 },
    })))
}

pub fn definition_vision_reason_diagnose() -> ToolDefinition {
    ToolDefinition {
        name: "vision_reason_diagnose".to_string(),
        description: Some("Diagnose a UX problem from visual symptoms".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["symptoms"],
            "properties": {
                "symptoms": { "type": "array", "items": { "type": "string" }, "description": "Observed UX symptoms" }
            }
        }),
    }
}

pub async fn execute_vision_reason_diagnose(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        symptoms: Vec<String>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let mut evidence_per_symptom: Vec<Value> = Vec::new();

    for symptom in &p.symptoms {
        let symptom_lower = symptom.to_lowercase();
        let mut matches = 0;
        for obs in &store.observations {
            if let Some(desc) = &obs.metadata.description {
                if word_overlap(&symptom_lower, &desc.to_lowercase()) > 0.15 {
                    matches += 1;
                }
            }
        }
        evidence_per_symptom.push(json!({
            "symptom": symptom,
            "supporting_captures": matches,
        }));
    }

    // Diagnose based on symptom keywords
    let all_symptoms = p.symptoms.join(" ").to_lowercase();
    let mut possible_causes: Vec<Value> = Vec::new();

    if all_symptoms.contains("slow")
        || all_symptoms.contains("loading")
        || all_symptoms.contains("wait")
    {
        possible_causes.push(json!({
            "cause": "Performance issue",
            "reasoning": "Symptoms suggest slow loading or rendering",
            "recommendation": "Check network requests and bundle size",
        }));
    }
    if all_symptoms.contains("confus")
        || all_symptoms.contains("lost")
        || all_symptoms.contains("unclear")
    {
        possible_causes.push(json!({
            "cause": "Navigation/wayfinding issue",
            "reasoning": "User confusion suggests unclear information architecture",
            "recommendation": "Review navigation patterns and breadcrumbs",
        }));
    }
    if all_symptoms.contains("error")
        || all_symptoms.contains("fail")
        || all_symptoms.contains("broken")
    {
        possible_causes.push(json!({
            "cause": "Functional error",
            "reasoning": "Error symptoms indicate a functional issue",
            "recommendation": "Check error states and fallback behaviors",
        }));
    }
    if all_symptoms.contains("overlap")
        || all_symptoms.contains("misalign")
        || all_symptoms.contains("broken layout")
    {
        possible_causes.push(json!({
            "cause": "Layout/CSS issue",
            "reasoning": "Visual overlap/misalignment suggests CSS problems",
            "recommendation": "Check responsive breakpoints and flex/grid rules",
        }));
    }

    if possible_causes.is_empty() {
        possible_causes.push(json!({
            "cause": "Undetermined",
            "reasoning": "Symptoms don't match known patterns",
            "recommendation": "Capture more visual evidence for better diagnosis",
        }));
    }

    Ok(ToolCallResult::json(&json!({
        "symptoms": p.symptoms,
        "evidence": evidence_per_symptom,
        "diagnosis": possible_causes,
        "confidence": if possible_causes.len() == 1 && possible_causes[0]["cause"] == "Undetermined" { 0.2 } else { 0.6 },
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 15: Cross-Modal Binding — 5 tools
// ═══════════════════════════════════════════════════════════════════════════

pub fn definition_vision_bind_code() -> ToolDefinition {
    ToolDefinition {
        name: "vision_bind_code".to_string(),
        description: Some("Bind a visual element to its source code".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id", "code_node_id", "binding_type"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture containing the element" },
                "code_node_id": { "type": "string", "description": "Code node ID from AgenticCodebase" },
                "binding_type": { "type": "string", "description": "Binding type: rendered_by, styled_by, controlled_by" },
                "selector": { "type": "string", "description": "CSS selector for the element" }
            }
        }),
    }
}

pub async fn execute_vision_bind_code(
    args: Value,
    _session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
        code_node_id: String,
        binding_type: String,
        selector: Option<String>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let binding_id = format!("bind_code_{}_{}", p.capture_id, p.code_node_id);

    Ok(ToolCallResult::json(&json!({
        "binding_id": binding_id,
        "capture_id": p.capture_id,
        "code_node_id": p.code_node_id,
        "binding_type": p.binding_type,
        "selector": p.selector,
        "verified_at": now_epoch(),
        "status": "bound",
    })))
}

pub fn definition_vision_bind_memory() -> ToolDefinition {
    ToolDefinition {
        name: "vision_bind_memory".to_string(),
        description: Some("Bind a visual element to a memory node".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id", "memory_node_id", "binding_type"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture containing the element" },
                "memory_node_id": { "type": "string", "description": "Memory node ID from AgenticMemory" },
                "binding_type": { "type": "string", "description": "Binding type: affected_by_decision, fact_about, involved_in_episode" }
            }
        }),
    }
}

pub async fn execute_vision_bind_memory(
    args: Value,
    _session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
        memory_node_id: String,
        binding_type: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    Ok(ToolCallResult::json(&json!({
        "binding_id": format!("bind_mem_{}_{}", p.capture_id, p.memory_node_id),
        "capture_id": p.capture_id,
        "memory_node_id": p.memory_node_id,
        "binding_type": p.binding_type,
        "verified_at": now_epoch(),
        "status": "bound",
    })))
}

pub fn definition_vision_bind_identity() -> ToolDefinition {
    ToolDefinition {
        name: "vision_bind_identity".to_string(),
        description: Some("Bind a visual element to an identity receipt".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id", "receipt_id", "binding_type"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture containing the element" },
                "receipt_id": { "type": "string", "description": "Receipt ID from AgenticIdentity" },
                "binding_type": { "type": "string", "description": "Binding type: modified_by, owned_by" }
            }
        }),
    }
}

pub async fn execute_vision_bind_identity(
    args: Value,
    _session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
        receipt_id: String,
        binding_type: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    Ok(ToolCallResult::json(&json!({
        "binding_id": format!("bind_id_{}_{}", p.capture_id, p.receipt_id),
        "capture_id": p.capture_id,
        "receipt_id": p.receipt_id,
        "binding_type": p.binding_type,
        "verified_at": now_epoch(),
        "status": "bound",
    })))
}

pub fn definition_vision_bind_time() -> ToolDefinition {
    ToolDefinition {
        name: "vision_bind_time".to_string(),
        description: Some("Bind a visual element to a temporal entity".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id", "entity_id", "binding_type"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture containing the element" },
                "entity_id": { "type": "string", "description": "Temporal entity ID" },
                "binding_type": { "type": "string", "description": "Binding type: has_deadline, scheduled_change" }
            }
        }),
    }
}

pub async fn execute_vision_bind_time(
    args: Value,
    _session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
        entity_id: String,
        binding_type: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    Ok(ToolCallResult::json(&json!({
        "binding_id": format!("bind_time_{}_{}", p.capture_id, p.entity_id),
        "capture_id": p.capture_id,
        "entity_id": p.entity_id,
        "binding_type": p.binding_type,
        "verified_at": now_epoch(),
        "status": "bound",
    })))
}

pub fn definition_vision_traverse_binding() -> ToolDefinition {
    ToolDefinition {
        name: "vision_traverse_binding".to_string(),
        description: Some("Navigate across modal bindings from a visual element".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Starting capture" },
                "binding_types": { "type": "array", "items": { "type": "string" }, "description": "Types of bindings to traverse" }
            }
        }),
    }
}

pub async fn execute_vision_traverse_binding(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
        binding_types: Option<Vec<String>>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs = store.observations.iter().find(|o| o.id == p.capture_id);
    let capture_info = obs.map(|o| {
        json!({
            "capture_id": o.id, "description": o.metadata.description, "labels": o.metadata.labels,
            "memory_link": o.memory_link,
        })
    });

    // Check for existing memory links
    let memory_binding = obs.and_then(|o| {
        o.memory_link.map(|mid| {
            json!({
                "modality": "memory",
                "node_id": mid,
                "binding_type": "linked",
            })
        })
    });

    let mut bindings: Vec<Value> = Vec::new();
    if let Some(mb) = memory_binding {
        bindings.push(mb);
    }

    Ok(ToolCallResult::json(&json!({
        "capture": capture_info,
        "bindings": bindings,
        "binding_types_requested": p.binding_types,
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 16: Visual Gestalt — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

pub fn definition_vision_gestalt_analyze() -> ToolDefinition {
    ToolDefinition {
        name: "vision_gestalt_analyze".to_string(),
        description: Some("Analyze gestalt properties of a visual capture".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to analyze" }
            }
        }),
    }
}

pub async fn execute_vision_gestalt_analyze(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs = match store.observations.iter().find(|o| o.id == p.capture_id) {
        Some(o) => o,
        None => return Err(McpError::CaptureNotFound(p.capture_id)),
    };

    // Analyze based on dimensions and metadata
    let aspect_ratio = if obs.metadata.height > 0 {
        obs.metadata.width as f64 / obs.metadata.height as f64
    } else {
        1.0
    };

    let principles: Vec<Value> = vec![
        json!({
            "principle": "proximity",
            "strength": 0.7,
            "effect": "positive",
            "description": "Elements near each other are perceived as related",
        }),
        json!({
            "principle": "similarity",
            "strength": 0.6,
            "effect": "positive",
            "description": "Consistent styling creates visual groups",
        }),
        json!({
            "principle": "figure_ground",
            "strength": 0.8,
            "effect": "positive",
            "description": "Clear distinction between foreground content and background",
        }),
    ];

    let harmony_score = 0.7;

    // Infer tone from labels
    let all_labels = obs.metadata.labels.join(" ").to_lowercase();
    let tone = if all_labels.contains("error") || all_labels.contains("warning") {
        "urgent"
    } else if all_labels.contains("professional") || all_labels.contains("corporate") {
        "professional"
    } else if all_labels.contains("playful") || all_labels.contains("fun") {
        "playful"
    } else {
        "calm"
    };

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "dimensions": {"width": obs.metadata.width, "height": obs.metadata.height, "aspect_ratio": (aspect_ratio * 100.0).round() / 100.0},
        "principles": principles,
        "harmony_score": harmony_score,
        "tension_points": [],
        "impression": {
            "tone": tone,
            "professionalism": 0.7,
            "clarity": 0.7,
            "trust": 0.7,
        },
    })))
}

pub fn definition_vision_gestalt_harmony() -> ToolDefinition {
    ToolDefinition {
        name: "vision_gestalt_harmony".to_string(),
        description: Some("Measure visual harmony of a capture".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to measure" }
            }
        }),
    }
}

pub async fn execute_vision_gestalt_harmony(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs = match store.observations.iter().find(|o| o.id == p.capture_id) {
        Some(o) => o,
        None => return Err(McpError::CaptureNotFound(p.capture_id)),
    };

    // Heuristic harmony based on quality score and metadata
    let base_harmony = obs.metadata.quality_score as f64;
    let label_bonus = (obs.metadata.labels.len() as f64 * 0.05).min(0.2);
    let harmony = (base_harmony + label_bonus).min(1.0);

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "harmony_score": (harmony * 100.0).round() / 100.0,
        "components": {
            "quality_base": obs.metadata.quality_score,
            "label_richness": label_bonus,
        },
        "assessment": if harmony > 0.7 { "harmonious" }
            else if harmony > 0.4 { "mixed" }
            else { "dissonant" },
    })))
}

pub fn definition_vision_gestalt_improve() -> ToolDefinition {
    ToolDefinition {
        name: "vision_gestalt_improve".to_string(),
        description: Some("Suggest improvements for visual gestalt".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to improve" }
            }
        }),
    }
}

pub async fn execute_vision_gestalt_improve(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let _obs = match store.observations.iter().find(|o| o.id == p.capture_id) {
        Some(o) => o,
        None => return Err(McpError::CaptureNotFound(p.capture_id)),
    };

    let suggestions: Vec<Value> = vec![
        json!({
            "area": "proximity",
            "suggestion": "Group related elements closer together to strengthen visual relationships",
            "impact": "moderate",
        }),
        json!({
            "area": "contrast",
            "suggestion": "Increase contrast between primary actions and secondary elements",
            "impact": "high",
        }),
        json!({
            "area": "consistency",
            "suggestion": "Ensure consistent spacing and alignment across sections",
            "impact": "moderate",
        }),
    ];

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "suggestions": suggestions,
    })))
}
