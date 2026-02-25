//! REPL slash-command dispatch and state management.

use super::commands;
use crate::types::VisionResult;
use std::path::PathBuf;

pub struct ReplState {
    pub file_path: Option<PathBuf>,
    pub json: bool,
}

impl ReplState {
    pub fn new() -> Self {
        Self {
            file_path: None,
            json: false,
        }
    }

    pub fn dispatch(&mut self, line: &str) -> VisionResult<()> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let cmd = parts[0].trim_start_matches('/');
        let args = &parts[1..];

        match cmd {
            "create" => self.cmd_create(args),
            "load" => self.cmd_load(args),
            "info" => self.cmd_info(),
            "capture" => self.cmd_capture(args),
            "query" => self.cmd_query(args),
            "similar" => self.cmd_similar(args),
            "compare" => self.cmd_compare(args),
            "diff" => self.cmd_diff(args),
            "health" => self.cmd_health(),
            "link" => self.cmd_link(args),
            "stats" => self.cmd_stats(),
            "export" => self.cmd_export(args),
            "json" => {
                self.json = !self.json;
                println!("JSON output: {}", if self.json { "on" } else { "off" });
                Ok(())
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
                Ok(())
            }
            "help" => {
                self.print_help();
                Ok(())
            }
            "exit" | "quit" => {
                println!("Goodbye.");
                std::process::exit(0);
            }
            _ => {
                let suggestion = suggest_command(cmd);
                if let Some(s) = suggestion {
                    println!("Unknown command '/{cmd}'. Did you mean '/{s}'?");
                } else {
                    println!("Unknown command '/{cmd}'. Type /help for available commands.");
                }
                Ok(())
            }
        }
    }

    fn require_file(&self) -> VisionResult<&PathBuf> {
        self.file_path.as_ref().ok_or_else(|| {
            crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file loaded. Use /create <path> or /load <path> first.",
            ))
        })
    }

    fn cmd_create(&mut self, args: &[&str]) -> VisionResult<()> {
        let path = args.first().ok_or_else(|| {
            crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: /create <path> [dimension]",
            ))
        })?;
        let dim: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(512);
        let p = PathBuf::from(path);
        commands::cmd_create(&p, dim)?;
        self.file_path = Some(p);
        Ok(())
    }

    fn cmd_load(&mut self, args: &[&str]) -> VisionResult<()> {
        let path = args.first().ok_or_else(|| {
            crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: /load <path>",
            ))
        })?;
        let p = PathBuf::from(path);
        let store = crate::storage::AvisReader::read_from_file(&p)?;
        println!("Loaded {} ({} observations)", p.display(), store.count());
        self.file_path = Some(p);
        Ok(())
    }

    fn cmd_info(&self) -> VisionResult<()> {
        commands::cmd_info(self.require_file()?, self.json)
    }

    fn cmd_capture(&self, args: &[&str]) -> VisionResult<()> {
        let p = self.require_file()?;
        let source = args.first().ok_or_else(|| {
            crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: /capture <image_path>",
            ))
        })?;
        commands::cmd_capture(p, source, Vec::new(), None, None, self.json)
    }

    fn cmd_query(&self, args: &[&str]) -> VisionResult<()> {
        let limit: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(20);
        commands::cmd_query(self.require_file()?, None, None, limit, self.json)
    }

    fn cmd_similar(&self, args: &[&str]) -> VisionResult<()> {
        let id: u64 = args.first().and_then(|s| s.parse().ok()).ok_or_else(|| {
            crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: /similar <capture_id> [top_k]",
            ))
        })?;
        let top_k: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10);
        commands::cmd_similar(self.require_file()?, id, top_k, 0.5, self.json)
    }

    fn cmd_compare(&self, args: &[&str]) -> VisionResult<()> {
        if args.len() < 2 {
            return Err(crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: /compare <id_a> <id_b>",
            )));
        }
        let id_a: u64 = args[0].parse().map_err(|_| {
            crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid capture ID",
            ))
        })?;
        let id_b: u64 = args[1].parse().map_err(|_| {
            crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid capture ID",
            ))
        })?;
        commands::cmd_compare(self.require_file()?, id_a, id_b, self.json)
    }

    fn cmd_diff(&self, args: &[&str]) -> VisionResult<()> {
        if args.len() < 2 {
            return Err(crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: /diff <id_a> <id_b>",
            )));
        }
        let id_a: u64 = args[0].parse().map_err(|_| {
            crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid capture ID",
            ))
        })?;
        let id_b: u64 = args[1].parse().map_err(|_| {
            crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid capture ID",
            ))
        })?;
        commands::cmd_diff(self.require_file()?, id_a, id_b, self.json)
    }

    fn cmd_health(&self) -> VisionResult<()> {
        commands::cmd_health(self.require_file()?, 168, 0.45, 20, self.json)
    }

    fn cmd_link(&self, args: &[&str]) -> VisionResult<()> {
        if args.len() < 2 {
            return Err(crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: /link <capture_id> <memory_node_id>",
            )));
        }
        let cid: u64 = args[0].parse().map_err(|_| {
            crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid capture ID",
            ))
        })?;
        let mid: u64 = args[1].parse().map_err(|_| {
            crate::types::VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid memory node ID",
            ))
        })?;
        commands::cmd_link(self.require_file()?, cid, mid, self.json)
    }

    fn cmd_stats(&self) -> VisionResult<()> {
        commands::cmd_stats(self.require_file()?, self.json)
    }

    fn cmd_export(&self, args: &[&str]) -> VisionResult<()> {
        let pretty = args.iter().any(|a| *a == "--pretty" || *a == "-p");
        commands::cmd_export(self.require_file()?, pretty)
    }

    fn print_help(&self) {
        println!("Available commands:");
        println!("  /create <path> [dim]        Create a new .avis file");
        println!("  /load <path>                Load an existing .avis file");
        println!("  /info                       Display file info");
        println!("  /capture <image>            Capture an image");
        println!("  /query [limit]              Search observations");
        println!("  /similar <id> [top_k]       Find similar captures");
        println!("  /compare <id_a> <id_b>      Compare two captures");
        println!("  /diff <id_a> <id_b>         Pixel-level diff");
        println!("  /health                     Quality report");
        println!("  /link <cap_id> <mem_id>     Link to memory node");
        println!("  /stats                      Aggregate statistics");
        println!("  /export [--pretty]          Export as JSON");
        println!("  /json                       Toggle JSON output");
        println!("  /clear                      Clear screen");
        println!("  /help                       Show this message");
        println!("  /exit                       Quit");
    }
}

fn suggest_command(input: &str) -> Option<&'static str> {
    const CMDS: &[&str] = &[
        "create", "load", "info", "capture", "query", "similar", "compare", "diff", "health",
        "link", "stats", "export", "json", "clear", "help", "exit",
    ];
    let mut best: Option<(&str, usize)> = None;
    for &cmd in CMDS {
        let dist = levenshtein(input, cmd);
        if dist <= 2 && (best.is_none() || dist < best.unwrap().1) {
            best = Some((cmd, dist));
        }
    }
    best.map(|(s, _)| s)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}
