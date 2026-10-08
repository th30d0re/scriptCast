//! Serializable pipeline decisions shared by the UI and detached worker.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    Render,
    Verify,
    Repair,
    Relayout,
    Stitch,
    TurnIndex,
}
impl Step {
    pub const ALL: [Self; 6] = [
        Self::Render,
        Self::Verify,
        Self::Repair,
        Self::Relayout,
        Self::Stitch,
        Self::TurnIndex,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Render => "Render",
            Self::Verify => "Verify",
            Self::Repair => "Repair",
            Self::Relayout => "Relayout",
            Self::Stitch => "Stitch MP3",
            Self::TurnIndex => "Turn index",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Ready,
    Running,
    RepairStopped,
    Failed(String),
    Complete,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pipeline {
    pub enabled: [bool; 6],
    pub current: Option<usize>,
    pub state: State,
    pub max_passes: usize,
    pub gap_ms: u64,
    pub failing: usize,
}
impl Default for Pipeline {
    fn default() -> Self {
        Self {
            enabled: [true; 6],
            current: None,
            state: State::Ready,
            max_passes: 3,
            gap_ms: 250,
            failing: 0,
        }
    }
}
impl Pipeline {
    pub fn next(&mut self) {
        self.current = ((self.current.map_or(0, |i| i + 1))..6).find(|i| self.enabled[*i]);
        self.state = if self.current.is_some() {
            State::Running
        } else {
            State::Complete
        };
    }
    pub fn finish(&mut self, exit: Option<i32>, error: Option<String>, failing: usize) {
        self.failing = failing;
        if let Some(message) = error {
            self.state = State::Failed(message);
        } else if exit != Some(0) {
            self.state = State::Failed(format!("Step exited with {exit:?}"));
        } else if self.current == Some(2) && failing > 0 {
            self.state = State::RepairStopped;
        } else {
            self.next();
        }
    }
    pub fn continue_anyway(&mut self) {
        if self.state == State::RepairStopped {
            self.next();
        }
    }
}
pub fn relayout_length(durations: impl IntoIterator<Item = u64>, gap_ms: u64) -> u64 {
    let mut count = 0u64;
    let mut total = 0u64;
    for duration in durations {
        total = total.saturating_add(duration);
        count += 1;
    }
    total.saturating_add(count.saturating_sub(1).saturating_mul(gap_ms))
}
