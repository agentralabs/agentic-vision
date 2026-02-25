//! Multi-context workspace manager for loading and querying multiple .avis files.

use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use agentic_vision::{AvisReader, VisualMemoryStore};

use crate::types::{McpError, McpResult};

/// Role of a context within a workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextRole {
    Primary,
    Secondary,
    Reference,
    Archive,
}

impl ContextRole {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "primary" => Some(Self::Primary),
            "secondary" => Some(Self::Secondary),
            "reference" => Some(Self::Reference),
            "archive" => Some(Self::Archive),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Reference => "reference",
            Self::Archive => "archive",
        }
    }
}

/// A loaded vision context within a workspace.
pub struct VisionContext {
    pub id: String,
    pub role: ContextRole,
    pub path: String,
    pub label: Option<String>,
    pub store: VisualMemoryStore,
}

/// A multi-vision workspace.
pub struct VisionWorkspace {
    pub id: String,
    pub name: String,
    pub contexts: Vec<VisionContext>,
    pub created_at: u64,
}

/// Cross-context match.
#[derive(Debug)]
pub struct CrossContextMatch {
    pub observation_id: u64,
    pub description: Option<String>,
    pub labels: Vec<String>,
    pub score: f32,
}

/// Cross-context result.
#[derive(Debug)]
pub struct CrossContextResult {
    pub context_id: String,
    pub context_role: ContextRole,
    pub matches: Vec<CrossContextMatch>,
}

/// Comparison.
#[derive(Debug)]
pub struct Comparison {
    pub item: String,
    pub found_in: Vec<String>,
    pub missing_from: Vec<String>,
    pub matches_per_context: Vec<(String, Vec<CrossContextMatch>)>,
}

/// Cross-reference.
#[derive(Debug)]
pub struct CrossReference {
    pub item: String,
    pub present_in: Vec<String>,
    pub absent_from: Vec<String>,
}

/// Manages multiple vision workspaces.
#[derive(Default)]
pub struct VisionWorkspaceManager {
    workspaces: HashMap<String, VisionWorkspace>,
    next_id: u64,
}

impl VisionWorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, name: &str) -> String {
        let id = format!("vws_{}", self.next_id);
        self.next_id += 1;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;
        self.workspaces.insert(
            id.clone(),
            VisionWorkspace {
                id: id.clone(),
                name: name.to_string(),
                contexts: Vec::new(),
                created_at: now,
            },
        );
        id
    }

    pub fn add_context(
        &mut self,
        workspace_id: &str,
        path: &str,
        role: ContextRole,
        label: Option<String>,
    ) -> McpResult<String> {
        let workspace = self.workspaces.get_mut(workspace_id).ok_or_else(|| {
            McpError::InvalidParams(format!("Workspace not found: {workspace_id}"))
        })?;

        let file_path = Path::new(path);
        if !file_path.exists() {
            return Err(McpError::InvalidParams(format!("File not found: {path}")));
        }

        let store = AvisReader::read_from_file(file_path)
            .map_err(|e| McpError::VisionError(format!("Failed to parse {path}: {e}")))?;

        let ctx_id = format!("vctx_{}_{}", workspace.contexts.len() + 1, workspace_id);

        workspace.contexts.push(VisionContext {
            id: ctx_id.clone(),
            role,
            path: path.to_string(),
            label: label.or_else(|| {
                file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            }),
            store,
        });

        Ok(ctx_id)
    }

    pub fn list(&self, workspace_id: &str) -> McpResult<&[VisionContext]> {
        let workspace = self.workspaces.get(workspace_id).ok_or_else(|| {
            McpError::InvalidParams(format!("Workspace not found: {workspace_id}"))
        })?;
        Ok(&workspace.contexts)
    }

    pub fn get(&self, workspace_id: &str) -> Option<&VisionWorkspace> {
        self.workspaces.get(workspace_id)
    }

    pub fn query_all(
        &self,
        workspace_id: &str,
        query: &str,
        max_per_context: usize,
    ) -> McpResult<Vec<CrossContextResult>> {
        let workspace = self.workspaces.get(workspace_id).ok_or_else(|| {
            McpError::InvalidParams(format!("Workspace not found: {workspace_id}"))
        })?;

        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut results = Vec::new();

        for ctx in &workspace.contexts {
            let mut matches = Vec::new();
            for obs in &ctx.store.observations {
                let mut score = 0.0f32;

                if let Some(ref desc) = obs.metadata.description {
                    let desc_lower = desc.to_lowercase();
                    let overlap = query_words
                        .iter()
                        .filter(|w| desc_lower.contains(**w))
                        .count();
                    score += overlap as f32 / query_words.len().max(1) as f32;
                }

                for label in &obs.metadata.labels {
                    if query_lower.contains(&label.to_lowercase()) {
                        score += 0.3;
                    }
                }

                if score > 0.0 {
                    matches.push(CrossContextMatch {
                        observation_id: obs.id,
                        description: obs.metadata.description.clone(),
                        labels: obs.metadata.labels.clone(),
                        score,
                    });
                }
            }

            matches.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            matches.truncate(max_per_context);

            results.push(CrossContextResult {
                context_id: ctx.id.clone(),
                context_role: ctx.role,
                matches,
            });
        }

        Ok(results)
    }

    pub fn compare(
        &self,
        workspace_id: &str,
        item: &str,
        max_per_context: usize,
    ) -> McpResult<Comparison> {
        let results = self.query_all(workspace_id, item, max_per_context)?;
        let workspace = self.workspaces.get(workspace_id).unwrap();

        let mut found_in = Vec::new();
        let mut missing_from = Vec::new();
        let mut matches_per_context = Vec::new();

        for (i, cr) in results.into_iter().enumerate() {
            let label = workspace.contexts[i]
                .label
                .clone()
                .unwrap_or_else(|| cr.context_id.clone());
            if cr.matches.is_empty() {
                missing_from.push(label);
            } else {
                found_in.push(label.clone());
                matches_per_context.push((label, cr.matches));
            }
        }

        Ok(Comparison {
            item: item.to_string(),
            found_in,
            missing_from,
            matches_per_context,
        })
    }

    pub fn cross_reference(&self, workspace_id: &str, item: &str) -> McpResult<CrossReference> {
        let c = self.compare(workspace_id, item, 5)?;
        Ok(CrossReference {
            item: c.item,
            present_in: c.found_in,
            absent_from: c.missing_from,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_role_roundtrip() {
        assert_eq!(ContextRole::from_str("primary"), Some(ContextRole::Primary));
        assert_eq!(ContextRole::from_str("ARCHIVE"), Some(ContextRole::Archive));
        assert_eq!(ContextRole::from_str("unknown"), None);
    }

    #[test]
    fn test_workspace_create() {
        let mut mgr = VisionWorkspaceManager::new();
        let id = mgr.create("test");
        assert!(id.starts_with("vws_"));
        assert!(mgr.get(&id).is_some());
    }

    #[test]
    fn test_workspace_not_found() {
        let mgr = VisionWorkspaceManager::new();
        assert!(mgr.list("nonexistent").is_err());
    }

    #[test]
    fn test_workspace_file_not_found() {
        let mut mgr = VisionWorkspaceManager::new();
        let id = mgr.create("test");
        assert!(mgr
            .add_context(&id, "/nonexistent.avis", ContextRole::Primary, None)
            .is_err());
    }
}
