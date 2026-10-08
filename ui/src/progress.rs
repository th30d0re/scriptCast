//! Incremental, newline-delimited progress from the Phase J CLI contract.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    time::Duration,
};

pub const TICK: Duration = Duration::from_millis(250);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    Start {
        tool: String,
        total: usize,
        ts: String,
    },
    TurnStart {
        turn_index: usize,
        turn_id: String,
        speaker_id: String,
        engine: String,
        n: usize,
        total: usize,
    },
    TurnDone {
        turn_index: usize,
        turn_id: String,
        n: usize,
        total: usize,
        seconds: f64,
    },
    Pass {
        pass: usize,
        max_passes: usize,
        failing: usize,
    },
    Result {
        passed: usize,
        failed: usize,
    },
    Stage {
        name: String,
    },
    Done {
        ok: bool,
        outputs: std::collections::BTreeMap<String, String>,
    },
    Error {
        message: String,
    },
}
#[derive(Default, Debug)]
pub struct Tail {
    length: u64,
    offset: u64,
    pending: Vec<u8>,
}
impl Tail {
    pub fn read(&mut self, path: &Path) -> Result<Vec<Event>, String> {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e.to_string()),
        };
        let length = file.metadata().map_err(|e| e.to_string())?.len();
        if length == self.length {
            return Ok(vec![]);
        }
        if length < self.length {
            *self = Self::default();
        }
        file.seek(SeekFrom::Start(self.offset))
            .map_err(|e| e.to_string())?;
        let count = file
            .read_to_end(&mut self.pending)
            .map_err(|e| e.to_string())?;
        self.offset += count as u64;
        self.length = length;
        let mut events = vec![];
        while let Some(end) = self.pending.iter().position(|b| *b == b'\n') {
            let line: Vec<_> = self.pending.drain(..=end).collect();
            if line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            events
                .push(serde_json::from_slice(&line).map_err(|e| format!("Invalid progress: {e}"))?);
        }
        Ok(events)
    }
}
#[derive(Clone, Default, Debug)]
pub struct Progress {
    pub total: usize,
    pub n: usize,
    pub current: Option<usize>,
    pub engine: String,
    pub completed: BTreeSet<usize>,
    pub seconds: f64,
    done_n: usize,
    pub terminal: Option<Result<(), String>>,
    pub failing: usize,
    pub pass: usize,
    pub stage: String,
    pub outputs: std::collections::BTreeMap<String, String>,
}
impl Progress {
    pub fn apply(&mut self, event: &Event) {
        match event {
            Event::Start { total, tool, .. } => {
                self.total = *total;
                self.stage.clone_from(tool);
            }
            Event::TurnStart {
                turn_index,
                n,
                total,
                engine,
                ..
            } => {
                self.current = Some(*turn_index);
                self.n = *n;
                self.total = *total;
                self.engine.clone_from(engine);
            }
            Event::TurnDone {
                turn_index,
                n,
                total,
                seconds,
                ..
            } => {
                if self.completed.insert(*turn_index) && seconds.is_finite() && *seconds >= 0.0 {
                    self.seconds += seconds;
                }
                self.n = *n;
                self.done_n = self.done_n.max(*n);
                self.total = *total;
                if self.current == Some(*turn_index) {
                    self.current = None;
                }
            }
            Event::Pass { pass, failing, .. } => {
                self.pass = *pass;
                self.failing = *failing;
            }
            Event::Result { failed, .. } => self.failing = *failed,
            Event::Stage { name } => self.stage.clone_from(name),
            Event::Done { ok, outputs } => {
                self.outputs.clone_from(outputs);
                self.terminal = Some(if *ok {
                    Ok(())
                } else {
                    Err("Job failed".into())
                })
            }
            Event::Error { message } => self.terminal = Some(Err(message.clone())),
        }
    }
    pub fn eta(&self) -> Option<Duration> {
        if self.completed.is_empty() {
            return None;
        }
        Some(Duration::from_secs_f64(
            self.seconds / self.completed.len() as f64
                * self.total.saturating_sub(self.done_n) as f64,
        ))
    }
    pub fn status(&self, elapsed: Duration) -> String {
        let eta = self
            .eta()
            .map(|d| format!("about {} left", clock(d)))
            .unwrap_or("estimating…".into());
        format!(
            "Rendering turn {} of {} · {} · {} elapsed · {eta}",
            self.n,
            self.total,
            self.engine,
            clock(elapsed)
        )
    }
}
pub fn clock(duration: Duration) -> String {
    let s = duration.as_secs();
    format!("{:02}:{:02}", s / 60, s % 60)
}
