//! CLI entry point for the `avis` command-line tool.

use std::path::PathBuf;
use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use agentic_vision::cli::commands;

#[derive(Parser)]
#[command(name = "avis", about = "AgenticVision CLI -- visual memory for AI agents")]
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
    Create { file: PathBuf, #[arg(long, default_value = "512")] dimension: u32 },
    /// Display information about an .avis file
    Info { file: PathBuf },
    /// Capture an image and add it to the store
    Capture {
        file: PathBuf, source: String,
        #[arg(long)] labels: Option<String>,
        #[arg(long)] description: Option<String>,
    },
    /// Search observations with filters
    Query {
        file: PathBuf,
        #[arg(long)] session: Option<u32>,
        #[arg(long)] labels: Option<String>,
        #[arg(long, default_value = "20")] limit: usize,
    },
    /// Find visually similar captures
    Similar {
        file: PathBuf, capture_id: u64,
        #[arg(long, default_value = "10")] top_k: usize,
        #[arg(long, default_value = "0.5")] min_similarity: f32,
    },
    /// Compare two captures by embedding similarity
    Compare { file: PathBuf, id_a: u64, id_b: u64 },
    /// Pixel-level diff between two captures
    Diff { file: PathBuf, id_a: u64, id_b: u64 },
    /// Quality and staleness health report
    Health {
        file: PathBuf,
        #[arg(long, default_value = "168")] stale_hours: u64,
        #[arg(long, default_value = "0.45")] low_quality: f32,
        #[arg(long, default_value = "20")] max_examples: usize,
    },
    /// Link a capture to a memory node
    Link { file: PathBuf, capture_id: u64, memory_node_id: u64 },
    /// Aggregate statistics
    Stats { file: PathBuf },
    /// Export observations as JSON
    Export { file: PathBuf, #[arg(long)] pretty: bool },
    /// Generate shell completion scripts
    Completions { shell: Shell },
}

fn main() {
    let cli = Cli::parse();
    let json = cli.format == "json";

    if cli.verbose { eprintln!("Verbose mode enabled"); }

    let result = match cli.command {
        None => match agentic_vision::cli::repl::run() {
            Ok(()) => return,
            Err(e) => { eprintln!("Error: {}", e); process::exit(1); }
        },
        Some(Commands::Completions { shell }) => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "avis", &mut std::io::stdout());
            Ok(())
        }
        Some(Commands::Create { file, dimension }) => commands::cmd_create(&file, dimension),
        Some(Commands::Info { file }) => commands::cmd_info(&file, json),
        Some(Commands::Capture { file, source, labels, description }) => {
            let label_vec = labels
                .map(|s| s.split(',').map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
                .unwrap_or_default();
            commands::cmd_capture(&file, &source, label_vec, description, cli.model.as_deref(), json)
        }
        Some(Commands::Query { file, session, labels, limit }) => {
            let label_vec = labels.map(|s| s.split(',').map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect());
            commands::cmd_query(&file, session, label_vec, limit, json)
        }
        Some(Commands::Similar { file, capture_id, top_k, min_similarity }) => commands::cmd_similar(&file, capture_id, top_k, min_similarity, json),
        Some(Commands::Compare { file, id_a, id_b }) => commands::cmd_compare(&file, id_a, id_b, json),
        Some(Commands::Diff { file, id_a, id_b }) => commands::cmd_diff(&file, id_a, id_b, json),
        Some(Commands::Health { file, stale_hours, low_quality, max_examples }) => commands::cmd_health(&file, stale_hours, low_quality, max_examples, json),
        Some(Commands::Link { file, capture_id, memory_node_id }) => commands::cmd_link(&file, capture_id, memory_node_id, json),
        Some(Commands::Stats { file }) => commands::cmd_stats(&file, json),
        Some(Commands::Export { file, pretty }) => commands::cmd_export(&file, pretty),
    };

    if let Err(e) = result { eprintln!("Error: {}", e); process::exit(1); }
}
