//! Interactive REPL for the avis CLI.

use rustyline::error::ReadlineError;
use rustyline::Editor;

use super::repl_commands::ReplState;
use super::repl_complete::{bind_keys, AvisHelper};
use crate::types::VisionResult;

fn history_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home).join(".avis_history")
}

pub fn run() -> VisionResult<()> {
    println!("avis -- AgenticVision interactive shell");
    println!("Type /help for commands, /exit to quit.\n");

    let helper = AvisHelper::new();
    let mut rl = Editor::new()
        .map_err(|e| crate::types::VisionError::Io(std::io::Error::other(e.to_string())))?;
    rl.set_helper(Some(helper));
    bind_keys(&mut rl);

    let hist = history_path();
    let _ = rl.load_history(&hist);

    let mut state = ReplState::new();

    loop {
        let prompt = if let Some(ref p) = state.file_path {
            format!(
                "avis({})> ",
                p.file_name().unwrap_or_default().to_string_lossy()
            )
        } else {
            "avis> ".to_string()
        };

        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(&line).ok();
                if let Err(e) = state.dispatch(&line) {
                    eprintln!("Error: {}", e);
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                println!("Goodbye.");
                break;
            }
            Err(e) => {
                eprintln!("Readline error: {}", e);
                break;
            }
        }
    }

    let _ = rl.save_history(&hist);
    Ok(())
}
