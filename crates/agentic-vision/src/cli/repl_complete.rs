//! Tab completion and hints for the avis REPL.

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::Validator;
use rustyline::{Cmd, ConditionalEventHandler, Context, Event, EventHandler, Helper, KeyEvent};

pub const COMMANDS: &[(&str, &str)] = &[
    ("create", "Create a new .avis file"),
    ("load", "Load an .avis file"),
    ("info", "Display info about the loaded file"),
    ("capture", "Capture an image from a file path"),
    ("query", "Search observations"),
    ("similar", "Find visually similar captures"),
    ("compare", "Compare two captures by embedding"),
    ("diff", "Pixel-level diff between two captures"),
    ("health", "Quality and staleness report"),
    ("link", "Link a capture to a memory node"),
    ("stats", "Aggregate statistics"),
    ("export", "Export observations as JSON"),
    ("clear", "Clear screen"),
    ("help", "Show available commands"),
    ("exit", "Quit the REPL"),
];

pub struct AvisHelper {
    hinter: HistoryHinter,
}

impl Default for AvisHelper {
    fn default() -> Self {
        Self {
            hinter: HistoryHinter::new(),
        }
    }
}

impl AvisHelper {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Completer for AvisHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let trimmed = line[..pos].trim_start();
        if !trimmed.starts_with('/') && !trimmed.is_empty() {
            return Ok((0, vec![]));
        }
        let prefix = trimmed.strip_prefix('/').unwrap_or_default();
        let matches: Vec<Pair> = COMMANDS
            .iter()
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(name, _desc)| Pair {
                display: format!("/{name}"),
                replacement: format!("/{name} "),
            })
            .collect();
        let start = pos - prefix.len() - if trimmed.starts_with('/') { 1 } else { 0 };
        Ok((start, matches))
    }
}

impl Hinter for AvisHelper {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for AvisHelper {}
impl Validator for AvisHelper {}
impl Helper for AvisHelper {}

pub struct TabCompleteOrAcceptHint;

use rustyline::EventContext;

impl ConditionalEventHandler for TabCompleteOrAcceptHint {
    fn handle(
        &self,
        _evt: &Event,
        _n: rustyline::RepeatCount,
        _positive: bool,
        _ctx: &EventContext,
    ) -> Option<Cmd> {
        Some(Cmd::Complete)
    }
}

pub fn bind_keys(editor: &mut rustyline::Editor<AvisHelper, rustyline::history::DefaultHistory>) {
    editor.bind_sequence(
        KeyEvent::from('\t'),
        EventHandler::Conditional(Box::new(TabCompleteOrAcceptHint)),
    );
}
