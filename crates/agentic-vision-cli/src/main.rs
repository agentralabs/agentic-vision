//! CLI entry point for the `avis` command-line tool.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use serde::{Deserialize, Serialize};

use agentic_vision::cli::commands;
use agentic_vision::storage::AvisReader;
use agentic_vision_mcp::session::workspace::{ContextRole, VisionWorkspaceManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceContext {
    path: String,
    role: String,
    label: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct WorkspaceState {
    workspaces: HashMap<String, Vec<WorkspaceContext>>,
}

fn state_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".agentic")
        .join("vision")
        .join("workspaces.json")
}

fn load_state() -> agentic_vision::VisionResult<WorkspaceState> {
    let path = state_path();
    if !path.exists() {
        return Ok(WorkspaceState::default());
    }
    let raw = std::fs::read_to_string(path)?;
    serde_json::from_str(&raw).map_err(|e| {
        agentic_vision::VisionError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    })
}

fn save_state(state: &WorkspaceState) -> agentic_vision::VisionResult<()> {
    let path = state_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let raw = serde_json::to_string_pretty(state).map_err(|e| {
        agentic_vision::VisionError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    })?;
    std::fs::write(path, raw)?;
    Ok(())
}

fn load_workspace_manager(
    state: &WorkspaceState,
    workspace: &str,
) -> agentic_vision::VisionResult<(VisionWorkspaceManager, String)> {
    let contexts = state.workspaces.get(workspace).ok_or_else(|| {
        agentic_vision::VisionError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("workspace '{}' not found", workspace),
        ))
    })?;

    let mut manager = VisionWorkspaceManager::new();
    let ws_id = manager.create(workspace);
    for ctx in contexts {
        let role = ContextRole::from_str(&ctx.role).unwrap_or(ContextRole::Primary);
        manager
            .add_context(&ws_id, &ctx.path, role, ctx.label.clone())
            .map_err(|e| {
                agentic_vision::VisionError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e.to_string(),
                ))
            })?;
    }

    Ok((manager, ws_id))
}

fn score_observation(query: &str, labels: &[String], description: Option<&str>) -> f32 {
    let query_lower = query.to_lowercase();
    let words: Vec<&str> = query_lower.split_whitespace().collect();
    let mut score = 0.0f32;

    if let Some(desc) = description {
        let desc_lower = desc.to_lowercase();
        let overlap = words.iter().filter(|w| desc_lower.contains(**w)).count();
        score += overlap as f32 / words.len().max(1) as f32;
    }

    for label in labels {
        if query_lower.contains(&label.to_lowercase()) {
            score += 0.3;
        }
    }

    score
}

fn suggest_observations(
    file: &Path,
    query: &str,
    limit: usize,
) -> agentic_vision::VisionResult<Vec<String>> {
    let store = AvisReader::read_from_file(file)?;
    let mut scored: Vec<(f32, String)> = Vec::new();
    for obs in &store.observations {
        let score = score_observation(
            query,
            &obs.metadata.labels,
            obs.metadata.description.as_deref(),
        );
        if score <= 0.0 {
            continue;
        }
        let text = obs
            .metadata
            .description
            .clone()
            .unwrap_or_else(|| format!("capture {}", obs.id));
        scored.push((score, text));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    Ok(scored.into_iter().map(|(_, s)| s).collect())
}

#[derive(Parser)]
#[command(
    name = "avis",
    about = "AgenticVision CLI -- visual memory for AI agents",
    version
)]
struct Cli {
    /// Output format: "text" (default) or "json"
    #[arg(long, default_value = "text")]
    format: String,
    /// Path to CLIP ONNX model
    #[arg(long)]
    model: Option<String>,
    /// Enable debug logging
    #[arg(long)]
    verbose: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new empty .avis file
    #[command(name = "init", alias = "create")]
    Create {
        file: PathBuf,
        #[arg(long, default_value = "512")]
        dimension: u32,
    },
    /// Display information about an .avis file
    Info { file: PathBuf },
    /// Capture an image and add it to the store
    Capture {
        file: PathBuf,
        source: String,
        #[arg(long)]
        labels: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    /// Search observations with filters
    #[command(name = "query")]
    Query {
        file: PathBuf,
        #[arg(long)]
        session: Option<u32>,
        #[arg(long)]
        labels: Option<String>,
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// Find visually similar captures
    Similar {
        file: PathBuf,
        capture_id: u64,
        #[arg(long, default_value = "10")]
        top_k: usize,
        #[arg(long, default_value = "0.5")]
        min_similarity: f32,
    },
    /// Compare two captures by embedding similarity
    Compare { file: PathBuf, id_a: u64, id_b: u64 },
    /// Pixel-level diff between two captures
    Diff { file: PathBuf, id_a: u64, id_b: u64 },
    /// Quality and staleness health report
    Health {
        file: PathBuf,
        #[arg(long, default_value = "168")]
        stale_hours: u64,
        #[arg(long, default_value = "0.45")]
        low_quality: f32,
        #[arg(long, default_value = "20")]
        max_examples: usize,
    },
    /// Link a capture to a memory node
    Link {
        file: PathBuf,
        capture_id: u64,
        memory_node_id: u64,
    },
    /// Aggregate statistics
    Stats { file: PathBuf },
    /// Export observations as JSON
    Export {
        file: PathBuf,
        #[arg(long)]
        pretty: bool,
    },
    /// Verify a claim has visual backing
    Ground {
        file: PathBuf,
        claim: String,
        #[arg(long, default_value = "0.3")]
        threshold: f32,
    },
    /// Return visual evidence for a query
    Evidence {
        file: PathBuf,
        query: String,
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Suggest similar captures for a phrase
    Suggest {
        file: PathBuf,
        query: String,
        #[arg(long, default_value = "5")]
        limit: usize,
    },
    /// Workspace operations across multiple vision files
    Workspace {
        #[command(subcommand)]
        subcommand: WorkspaceCommands,
    },
    /// Generate shell completion scripts
    Completions { shell: Shell },
}

#[derive(Subcommand)]
enum WorkspaceCommands {
    /// Create a workspace
    Create { name: String },
    /// Add a vision file to a workspace
    Add {
        workspace: String,
        file: PathBuf,
        #[arg(long, default_value = "primary")]
        role: String,
        #[arg(long)]
        label: Option<String>,
    },
    /// List files in a workspace
    List { workspace: String },
    /// Query across all files in a workspace
    Query {
        workspace: String,
        query: String,
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Compare an item across workspace contexts
    Compare {
        workspace: String,
        item: String,
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Cross-reference an item across workspace contexts
    Xref { workspace: String, item: String },
}

fn main() {
    let cli = Cli::parse();
    let json = cli.format == "json";

    if cli.verbose {
        eprintln!("Verbose mode enabled");
    }

    let result = match cli.command {
        None => match agentic_vision::cli::repl::run() {
            Ok(()) => return,
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Completions { shell }) => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "avis", &mut std::io::stdout());
            Ok(())
        }
        Some(Commands::Create { file, dimension }) => commands::cmd_create(&file, dimension),
        Some(Commands::Info { file }) => commands::cmd_info(&file, json),
        Some(Commands::Capture {
            file,
            source,
            labels,
            description,
        }) => {
            let label_vec = labels
                .map(|s| {
                    s.split(',')
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            commands::cmd_capture(
                &file,
                &source,
                label_vec,
                description,
                cli.model.as_deref(),
                json,
            )
        }
        Some(Commands::Query {
            file,
            session,
            labels,
            limit,
        }) => {
            let label_vec = labels.map(|s| {
                s.split(',')
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect()
            });
            commands::cmd_query(&file, session, label_vec, limit, json)
        }
        Some(Commands::Similar {
            file,
            capture_id,
            top_k,
            min_similarity,
        }) => commands::cmd_similar(&file, capture_id, top_k, min_similarity, json),
        Some(Commands::Compare { file, id_a, id_b }) => {
            commands::cmd_compare(&file, id_a, id_b, json)
        }
        Some(Commands::Diff { file, id_a, id_b }) => commands::cmd_diff(&file, id_a, id_b, json),
        Some(Commands::Health {
            file,
            stale_hours,
            low_quality,
            max_examples,
        }) => commands::cmd_health(&file, stale_hours, low_quality, max_examples, json),
        Some(Commands::Link {
            file,
            capture_id,
            memory_node_id,
        }) => commands::cmd_link(&file, capture_id, memory_node_id, json),
        Some(Commands::Stats { file }) => commands::cmd_stats(&file, json),
        Some(Commands::Export { file, pretty }) => commands::cmd_export(&file, pretty),
        Some(Commands::Ground {
            file,
            claim,
            threshold,
        }) => (|| -> agentic_vision::VisionResult<()> {
            let store = AvisReader::read_from_file(&file)?;
            let mut evidence = Vec::new();
            for obs in &store.observations {
                let score = score_observation(
                    &claim,
                    &obs.metadata.labels,
                    obs.metadata.description.as_deref(),
                );
                if score >= threshold {
                    evidence.push(serde_json::json!({
                        "id": obs.id,
                        "score": score,
                        "labels": obs.metadata.labels,
                        "description": obs.metadata.description,
                    }));
                }
            }

            if evidence.is_empty() {
                let suggestions = suggest_observations(&file, &claim, 5)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "ungrounded",
                            "claim": claim,
                            "suggestions": suggestions
                        }))
                        .unwrap_or_default()
                    );
                } else {
                    println!("Status: ungrounded");
                    println!("Claim: {}", claim);
                    if suggestions.is_empty() {
                        println!("Suggestions: none");
                    } else {
                        println!("Suggestions:");
                        for s in suggestions {
                            println!("  - {}", s);
                        }
                    }
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "status": "verified",
                        "claim": claim,
                        "evidence_count": evidence.len(),
                        "evidence": evidence
                    }))
                    .unwrap_or_default()
                );
            } else {
                println!("Status: verified");
                println!("Claim: {}", claim);
                println!("Evidence count: {}", evidence.len());
                for row in evidence {
                    let id = row.get("id").and_then(|v| v.as_u64()).unwrap_or_default();
                    let score = row
                        .get("score")
                        .and_then(|v| v.as_f64())
                        .unwrap_or_default();
                    let desc = row
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<no description>");
                    println!("  - [{}] score={:.3} {}", id, score, desc);
                }
            }
            Ok(())
        })(),
        Some(Commands::Evidence { file, query, limit }) => {
            (|| -> agentic_vision::VisionResult<()> {
                let store = AvisReader::read_from_file(&file)?;
                let mut evidence = Vec::new();
                for obs in &store.observations {
                    let score = score_observation(
                        &query,
                        &obs.metadata.labels,
                        obs.metadata.description.as_deref(),
                    );
                    if score > 0.0 {
                        evidence.push(serde_json::json!({
                            "id": obs.id,
                            "score": score,
                            "labels": obs.metadata.labels,
                            "description": obs.metadata.description,
                        }));
                    }
                }
                evidence.sort_by(|a, b| {
                    let sa = a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let sb = b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                });
                evidence.truncate(limit);

                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "query": query,
                            "count": evidence.len(),
                            "evidence": evidence
                        }))
                        .unwrap_or_default()
                    );
                } else if evidence.is_empty() {
                    println!("No evidence found.");
                } else {
                    println!("Evidence for {:?}:", query);
                    for row in evidence {
                        let id = row.get("id").and_then(|v| v.as_u64()).unwrap_or_default();
                        let score = row
                            .get("score")
                            .and_then(|v| v.as_f64())
                            .unwrap_or_default();
                        let desc = row
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("<no description>");
                        println!("  - [{}] score={:.3} {}", id, score, desc);
                    }
                }
                Ok(())
            })()
        }
        Some(Commands::Suggest { file, query, limit }) => {
            (|| -> agentic_vision::VisionResult<()> {
                let suggestions = suggest_observations(&file, &query, limit)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "query": query,
                            "suggestions": suggestions
                        }))
                        .unwrap_or_default()
                    );
                } else if suggestions.is_empty() {
                    println!("No suggestions found.");
                } else {
                    println!("Suggestions:");
                    for s in suggestions {
                        println!("  - {}", s);
                    }
                }
                Ok(())
            })()
        }
        Some(Commands::Workspace { subcommand }) => (|| -> agentic_vision::VisionResult<()> {
            match subcommand {
                WorkspaceCommands::Create { name } => {
                    let mut state = load_state()?;
                    state.workspaces.entry(name.clone()).or_default();
                    save_state(&state)?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "workspace": name,
                                "created": true
                            }))
                            .unwrap_or_default()
                        );
                    } else {
                        println!("Created workspace '{}'", name);
                    }
                    Ok(())
                }
                WorkspaceCommands::Add {
                    workspace,
                    file,
                    role,
                    label,
                } => {
                    let mut state = load_state()?;
                    let contexts = state.workspaces.entry(workspace.clone()).or_default();
                    let file_path = file.to_string_lossy().to_string();
                    if !contexts.iter().any(|ctx| ctx.path == file_path) {
                        contexts.push(WorkspaceContext {
                            path: file_path.clone(),
                            role: role.to_ascii_lowercase(),
                            label,
                        });
                        save_state(&state)?;
                    }
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "workspace": workspace,
                                "path": file_path,
                                "added": true
                            }))
                            .unwrap_or_default()
                        );
                    } else {
                        println!("Added {} to workspace '{}'", file.display(), workspace);
                    }
                    Ok(())
                }
                WorkspaceCommands::List { workspace } => {
                    let state = load_state()?;
                    let contexts = state.workspaces.get(&workspace).ok_or_else(|| {
                        agentic_vision::VisionError::Io(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("workspace '{}' not found", workspace),
                        ))
                    })?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "workspace": workspace,
                                "contexts": contexts
                            }))
                            .unwrap_or_default()
                        );
                    } else {
                        println!("Workspace '{}':", workspace);
                        for ctx in contexts {
                            println!(
                                "  - {} (role={}, label={})",
                                ctx.path,
                                ctx.role,
                                ctx.label.clone().unwrap_or_else(|| "-".to_string())
                            );
                        }
                    }
                    Ok(())
                }
                WorkspaceCommands::Query {
                    workspace,
                    query,
                    limit,
                } => {
                    let state = load_state()?;
                    let (manager, ws_id) = load_workspace_manager(&state, &workspace)?;
                    let results = manager.query_all(&ws_id, &query, limit).map_err(|e| {
                        agentic_vision::VisionError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            e.to_string(),
                        ))
                    })?;
                    if json {
                        let rows: Vec<_> = results
                            .iter()
                            .map(|r| {
                                serde_json::json!({
                                    "context_id": r.context_id,
                                    "role": r.context_role.label(),
                                    "matches": r.matches.iter().map(|m| serde_json::json!({
                                        "observation_id": m.observation_id,
                                        "score": m.score,
                                        "labels": m.labels,
                                        "description": m.description,
                                    })).collect::<Vec<_>>()
                                })
                            })
                            .collect();
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "workspace": workspace,
                                "query": query,
                                "results": rows
                            }))
                            .unwrap_or_default()
                        );
                    } else {
                        println!("Workspace query '{}':", query);
                        for r in results {
                            println!("  Context {} ({})", r.context_id, r.context_role.label());
                            for m in r.matches {
                                println!(
                                    "    - [{}] score={:.3} {:?}",
                                    m.observation_id, m.score, m.labels
                                );
                            }
                        }
                    }
                    Ok(())
                }
                WorkspaceCommands::Compare {
                    workspace,
                    item,
                    limit,
                } => {
                    let state = load_state()?;
                    let (manager, ws_id) = load_workspace_manager(&state, &workspace)?;
                    let comparison = manager.compare(&ws_id, &item, limit).map_err(|e| {
                        agentic_vision::VisionError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            e.to_string(),
                        ))
                    })?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "workspace": workspace,
                                "item": comparison.item,
                                "found_in": comparison.found_in,
                                "missing_from": comparison.missing_from
                            }))
                            .unwrap_or_default()
                        );
                    } else {
                        println!("Found in: {:?}", comparison.found_in);
                        println!("Missing from: {:?}", comparison.missing_from);
                    }
                    Ok(())
                }
                WorkspaceCommands::Xref { workspace, item } => {
                    let state = load_state()?;
                    let (manager, ws_id) = load_workspace_manager(&state, &workspace)?;
                    let xref = manager.cross_reference(&ws_id, &item).map_err(|e| {
                        agentic_vision::VisionError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            e.to_string(),
                        ))
                    })?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "workspace": workspace,
                                "item": xref.item,
                                "present_in": xref.present_in,
                                "absent_from": xref.absent_from
                            }))
                            .unwrap_or_default()
                        );
                    } else {
                        println!("Present in: {:?}", xref.present_in);
                        println!("Absent from: {:?}", xref.absent_from);
                    }
                    Ok(())
                }
            }
        })(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
