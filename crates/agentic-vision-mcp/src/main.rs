//! AgenticVision MCP Server — entry point.

use std::sync::Arc;
use tokio::sync::Mutex;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use agentic_vision_mcp::config::resolve_vision_path;
use agentic_vision_mcp::protocol::ProtocolHandler;
use agentic_vision_mcp::session::VisionSessionManager;
use agentic_vision_mcp::tools::ToolRegistry;
use agentic_vision_mcp::transport::StdioTransport;

#[derive(Parser)]
#[command(
    name = "agentic-vision-mcp",
    about = "MCP server for AgenticVision — universal LLM access to persistent visual memory",
    version
)]
struct Cli {
    /// Path to .avis vision file.
    #[arg(short, long)]
    vision: Option<String>,

    /// Path to CLIP ONNX model.
    #[arg(long)]
    model: Option<String>,

    /// Log level (trace, debug, info, warn, error).
    #[arg(long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start MCP server over stdio (default).
    Serve {
        /// Path to .avis vision file.
        #[arg(short, long)]
        vision: Option<String>,

        /// Path to CLIP ONNX model.
        #[arg(long)]
        model: Option<String>,

        /// Log level (trace, debug, info, warn, error).
        #[arg(long)]
        log_level: Option<String>,
    },

    /// Start MCP server over HTTP.
    #[cfg(feature = "sse")]
    ServeHttp {
        /// Listen address (host:port).
        #[arg(long, default_value = "127.0.0.1:3100")]
        addr: String,

        /// Path to .avis vision file (single-user mode).
        #[arg(short, long)]
        vision: Option<String>,

        /// Path to CLIP ONNX model.
        #[arg(long)]
        model: Option<String>,

        /// Log level (trace, debug, info, warn, error).
        #[arg(long)]
        log_level: Option<String>,

        /// Bearer token for authentication.
        /// Also reads from AGENTIC_TOKEN env var.
        #[arg(long)]
        token: Option<String>,

        /// Enable multi-tenant mode (per-user vision files).
        #[arg(long)]
        multi_tenant: bool,

        /// Data directory for multi-tenant vision files.
        /// Each user gets {data-dir}/{user-id}.avis.
        #[arg(long)]
        data_dir: Option<String>,
    },

    /// Validate a .avis vision file.
    Validate,

    /// Print server capabilities as JSON.
    Info,

    /// Generate shell completion scripts.
    ///
    /// Examples:
    ///   agentic-vision-mcp completions bash > ~/.local/share/bash-completion/completions/agentic-vision-mcp
    ///   agentic-vision-mcp completions zsh > ~/.zfunc/_agentic-vision-mcp
    Completions {
        /// Shell type (bash, zsh, fish, powershell, elvish).
        shell: Shell,
    },

    /// Launch interactive REPL mode.
    Repl,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cli.log_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    match cli.command.unwrap_or(Commands::Serve {
        vision: None,
        model: None,
        log_level: None,
    }) {
        Commands::Serve {
            vision,
            model,
            log_level: _,
        } => {
            let effective_vision = vision.or(cli.vision);
            let effective_model = model.or(cli.model);
            let vision_path = resolve_vision_path(effective_vision.as_deref());
            let session = VisionSessionManager::open(&vision_path, effective_model.as_deref())?;
            let session = Arc::new(Mutex::new(session));

            // Ghost Writer: background sync to Claude, Cursor, Windsurf, Cody
            let _ghost_writer_task =
                agentic_vision_mcp::ghost_bridge::spawn_ghost_writer(session.clone());

            let handler = ProtocolHandler::new(session);
            let transport = StdioTransport::new(handler);
            transport.run().await?;
        }

        #[cfg(feature = "sse")]
        Commands::ServeHttp {
            addr,
            vision,
            model,
            log_level: _,
            token,
            multi_tenant,
            data_dir,
        } => {
            use agentic_vision_mcp::session::tenant::VisionTenantRegistry;
            use agentic_vision_mcp::transport::sse::{ServerMode, SseTransport};

            // Resolve token: CLI flag > env var
            let effective_token = token.or_else(|| std::env::var("AGENTIC_TOKEN").ok());

            let server_mode = if multi_tenant {
                let dir = data_dir.unwrap_or_else(|| {
                    eprintln!("Error: --data-dir is required when using --multi-tenant");
                    std::process::exit(1);
                });
                let dir = std::path::PathBuf::from(&dir);
                let effective_model = model.or(cli.model);
                tracing::info!("AgenticVision MCP server (multi-tenant)");
                tracing::info!("Data dir: {}", dir.display());
                ServerMode::MultiTenant {
                    data_dir: dir.clone(),
                    model_path: effective_model,
                    registry: Arc::new(Mutex::new(VisionTenantRegistry::new(&dir, None))),
                }
            } else {
                let effective_vision = vision.or(cli.vision);
                let effective_model = model.or(cli.model);
                let vision_path = resolve_vision_path(effective_vision.as_deref());
                tracing::info!("AgenticVision MCP server");
                tracing::info!("Vision: {vision_path}");
                let session = VisionSessionManager::open(&vision_path, effective_model.as_deref())?;
                let session = Arc::new(Mutex::new(session));
                let handler = ProtocolHandler::new(session);
                ServerMode::Single(Arc::new(handler))
            };

            if effective_token.is_some() {
                tracing::info!("Auth: bearer token required");
            }

            let transport = SseTransport::with_config(effective_token, server_mode);
            transport.run(&addr).await?;
        }

        Commands::Validate => {
            let vision_path = resolve_vision_path(cli.vision.as_deref());
            match VisionSessionManager::open(&vision_path, None) {
                Ok(session) => {
                    let store = session.store();
                    println!("Valid vision file: {vision_path}");
                    println!("  Captures: {}", store.count());
                    println!("  Embedding dim: {}", store.embedding_dim);
                    println!("  Sessions: {}", store.session_count);
                }
                Err(e) => {
                    eprintln!("Invalid vision file: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Info => {
            let capabilities = agentic_vision_mcp::types::InitializeResult::default_result();
            let tools = ToolRegistry::list_tools();
            let info = serde_json::json!({
                "server": capabilities.server_info,
                "protocol_version": capabilities.protocol_version,
                "capabilities": capabilities.capabilities,
                "tools": tools.iter().map(|t| &t.name).collect::<Vec<_>>(),
                "tool_count": tools.len(),
            });
            println!("{}", serde_json::to_string_pretty(&info)?);
        }

        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(
                shell,
                &mut cmd,
                "agentic-vision-mcp",
                &mut std::io::stdout(),
            );
        }

        Commands::Repl => {
            agentic_vision_mcp::repl::run()?;
        }
    }

    Ok(())
}
