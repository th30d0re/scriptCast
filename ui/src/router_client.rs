//! Minimal router client shared by the AI sidebar (R7) and stage scroll view (R8).
//! The Laya router (R1) is not built yet; `JsonlQueueClient` only queues.
use std::io::Write as _;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// How a request was entered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Typed,
    Voice,
    Selection,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionContext {
    pub episode: String,
    pub turn_ids: Vec<String>,
    pub cue_ids: Vec<String>,
    pub frame_paths: Vec<PathBuf>,
}

/// One submission from the sidebar.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouterRequest {
    pub id: u64,
    pub text: String,
    pub source: Source,
    /// Episode directory open when the request was made, if any.
    pub episode: Option<PathBuf>,
    pub submitted_at: String,
}

impl RouterRequest {
    pub fn selection(id: u64, context: &SelectionContext, episode: Option<PathBuf>, submitted_at: String) -> Self {
        Self { id, text: serde_json::to_string(context).unwrap_or_default(), source: Source::Selection, episode, submitted_at }
    }

    pub fn selection_context(&self) -> Option<SelectionContext> {
        (self.source == Source::Selection).then(|| serde_json::from_str(&self.text).ok()).flatten()
    }
}

pub fn now_string() -> String {
    if crate::FIXED_CLOCK.load(std::sync::atomic::Ordering::Relaxed) {
        return "2026-01-01T00:00:00Z".into();
    }
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub type JsonlQueue = JsonlQueueClient;

/// What the client did with a request.
#[derive(Clone, Debug, PartialEq)]
pub enum Delivery {
    Queued,
    Failed(String),
}

/// The sidebar's only dependency on the router.
pub trait RouterClient {
    fn submit(&mut self, request: &RouterRequest) -> Delivery;
    /// Short label shown under the composer.
    fn describe(&self) -> String;
}

/// Appends each request as one JSON line. R1's router will consume the file.
pub struct JsonlQueueClient {
    pub path: PathBuf,
}

impl JsonlQueueClient {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
    /// `$SCRIPTCAST_ROUTER_QUEUE`, else `<data dir>/router/queue.jsonl`.
    pub fn default_path() -> PathBuf {
        if let Some(path) = std::env::var_os("SCRIPTCAST_ROUTER_QUEUE") {
            return PathBuf::from(path);
        }
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default();
        home.join("Library/Application Support/ScriptCast/router/queue.jsonl")
    }
}

impl RouterClient for JsonlQueueClient {
    fn submit(&mut self, request: &RouterRequest) -> Delivery {
        let write = || -> std::io::Result<()> {
            if let Some(parent) = self.path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let line = serde_json::to_string(request).map_err(std::io::Error::other)?;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;
            writeln!(file, "{line}")
        };
        match write() {
            Ok(()) => Delivery::Queued,
            Err(error) => Delivery::Failed(error.to_string()),
        }
    }
    fn describe(&self) -> String {
        "queue only, router not built (R1)".into()
    }
}

/// Keeps requests in memory; used by the headless harness and tests.
#[derive(Default)]
pub struct MemoryClient {
    pub requests: Vec<RouterRequest>,
}

impl RouterClient for MemoryClient {
    fn submit(&mut self, request: &RouterRequest) -> Delivery {
        self.requests.push(request.clone());
        Delivery::Queued
    }
    fn describe(&self) -> String {
        "queue only, router not built (R1)".into()
    }
}
