//! Reviewer decision helpers for the Verification inspector (Phase H5).
use std::path::Path;

/// Outcomes stored in `review_log.jsonl` / `scriptcast-review add --outcome`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    RecognizerMisheard,
    AudioWrong,
    ScriptNeedsCorrection,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RecognizerMisheard => "recognizer_misheard",
            Self::AudioWrong => "audio_wrong",
            Self::ScriptNeedsCorrection => "script_needs_correction",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::RecognizerMisheard => "Whisper was wrong — recognition error",
            Self::AudioWrong => "Audio actually wrong",
            Self::ScriptNeedsCorrection => "Expected script needs correction",
        }
    }

    pub fn explanation(self) -> &'static str {
        match self {
            Self::RecognizerMisheard => {
                "Audio acceptable after manual listening; record a reviewer exception."
            }
            Self::AudioWrong => "The spoken audio does not match the intended script.",
            Self::ScriptNeedsCorrection => {
                "Revise the reference; keep audio unchanged until a separate decision."
            }
        }
    }

    /// Short title for discrepancy-view outcome cards (Figma 48:3588).
    pub fn card_title(self) -> &'static str {
        match self {
            Self::AudioWrong => "Audio is wrong",
            Self::ScriptNeedsCorrection => "Correct script only",
            Self::RecognizerMisheard => "Recognition error",
        }
    }

    /// One-line detail under [`Self::card_title`].
    pub fn card_detail(self) -> &'static str {
        match self {
            Self::AudioWrong => "Regenerate only after confirmation",
            Self::ScriptNeedsCorrection => "Edit the script; keep rendered audio",
            Self::RecognizerMisheard => "Whisper misheard; record a reviewer exception",
        }
    }

    pub fn all() -> [Self; 3] {
        [
            Self::AudioWrong,
            Self::ScriptNeedsCorrection,
            Self::RecognizerMisheard,
        ]
    }
}

/// Shared by the Verification inspector and the discrepancy review view.
pub fn choose_outcome(form: &mut DecisionForm, outcome: Outcome) {
    form.outcome = outcome;
    form.saved = false;
}

/// One expected/heard word pair the operator may accept for future verifies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptPair {
    pub word: String,
    pub heard: String,
    pub checked: bool,
    /// Accepting this would let a very common word pass in every turn of the
    /// project (operator, 2026-10-05: "in" heard as "it" slipped in this way).
    pub risky: bool,
}

/// Short, frequent words an accepted hearing would wave through project-wide.
const COMMON_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "from", "had", "has", "have",
    "he", "her", "his", "i", "if", "in", "into", "is", "it", "its", "me", "my", "no", "not", "of",
    "on", "or", "our", "she", "so", "that", "the", "their", "them", "then", "there", "they",
    "this", "to", "up", "us", "was", "we", "were", "what", "when", "which", "who", "will", "with",
    "you", "your",
];

/// True when accepting `word` heard as `heard` is likely to hide real errors.
pub fn risky_accept(word: &str, heard: &str) -> bool {
    let common = |w: &str| w.chars().count() <= 2 || COMMON_WORDS.contains(&w);
    common(word) || common(heard)
}

#[derive(Clone, Debug)]
pub struct DecisionForm {
    pub turn_id: String,
    pub outcome: Outcome,
    pub correction: String,
    pub accepts: Vec<AcceptPair>,
    pub saved: bool,
    pub notice: Option<String>,
    pub busy: bool,
}

impl DecisionForm {
    pub fn for_turn(turn_id: &str, expected: &str, heard: &str) -> Self {
        let pairs = replace_pairs(expected, heard);
        // Pre-tick only a lone, uncommon pair. Several pairs from one span, or a
        // common word, must be chosen deliberately.
        let single = pairs.len() == 1;
        let accepts = pairs
            .into_iter()
            .map(|(word, heard)| {
                let risky = risky_accept(&word, &heard);
                AcceptPair {
                    checked: single && !risky,
                    word,
                    heard,
                    risky,
                }
            })
            .collect();
        Self {
            turn_id: turn_id.to_owned(),
            outcome: Outcome::RecognizerMisheard,
            correction: String::new(),
            accepts,
            saved: false,
            notice: None,
            busy: false,
        }
    }

    /// Single-word accept UI when exactly one differing pair exists.
    pub fn single_accept(&self) -> Option<&AcceptPair> {
        (self.accepts.len() == 1).then(|| &self.accepts[0])
    }

    pub fn calibrate_hint(&self) -> Option<String> {
        if self.outcome != Outcome::AudioWrong {
            return None;
        }
        let pair = self.single_accept()?;
        Some(format!(
            "Try a respelling: scriptcast-calibrate record {} --candidates <respelling>",
            pair.word
        ))
    }
}

/// `project_dir` is the parent of the configured voices file.
pub fn project_dir_from_voices(voices: &Path) -> Option<std::path::PathBuf> {
    voices.parent().map(|p| p.to_path_buf())
}

fn normalize_words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Replace opcodes only: pairs of (expected_word, heard_word) that differ.
pub fn replace_pairs(expected: &str, heard: &str) -> Vec<(String, String)> {
    let a = normalize_words(expected);
    let b = normalize_words(heard);
    let mut pairs = Vec::new();
    for (tag, i1, i2, j1, j2) in opcodes(&a, &b) {
        if tag != Tag::Replace || (i2 - i1) != (j2 - j1) {
            continue;
        }
        for (di, dj) in (i1..i2).zip(j1..j2) {
            if a[di] != b[dj] {
                pairs.push((a[di].clone(), b[dj].clone()));
            }
        }
    }
    pairs
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tag {
    Equal,
    Replace,
    Delete,
    Insert,
}

/// Minimal SequenceMatcher-style opcodes via LCS.
fn opcodes(a: &[String], b: &[String]) -> Vec<(Tag, usize, usize, usize, usize)> {
    let (n, m) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            let start_i = i;
            let start_j = j;
            while i < n && j < m && a[i] == b[j] {
                i += 1;
                j += 1;
            }
            out.push((Tag::Equal, start_i, i, start_j, j));
        } else if j < m && (i == n || dp[i][j + 1] >= dp[i + 1][j]) {
            let start_j = j;
            while j < m && (i == n || (i < n && a[i] != b[j] && dp[i][j + 1] >= dp[i + 1][j])) {
                j += 1;
                if i < n && j < m && a[i] == b[j] {
                    break;
                }
            }
            // Prefer replace when both sides advance.
            if i < n && start_j < j {
                // Look ahead: consume a matching-length delete+insert as replace.
                let start_i = i;
                let take = (j - start_j).min(n - i);
                // If LCS says we should delete from a instead, do that.
                if take > 0 && dp[i + 1][start_j] < dp[i][start_j + 1] {
                    // insert only already handled by advancing j
                    out.push((Tag::Insert, i, i, start_j, j));
                } else if take > 0 {
                    i += take;
                    out.push((Tag::Replace, start_i, i, start_j, start_j + take));
                    if start_j + take < j {
                        out.push((Tag::Insert, i, i, start_j + take, j));
                    }
                } else {
                    out.push((Tag::Insert, i, i, start_j, j));
                }
            } else {
                out.push((Tag::Insert, i, i, start_j, j));
            }
        } else if i < n {
            let start_i = i;
            while i < n && (j == m || (j < m && a[i] != b[j] && dp[i + 1][j] >= dp[i][j + 1])) {
                i += 1;
                if i < n && j < m && a[i] == b[j] {
                    break;
                }
            }
            out.push((Tag::Delete, start_i, i, j, j));
        } else {
            break;
        }
    }
    out
}

/// Inputs for `scriptcast-review add` aside from the form state.
pub struct AddRequest<'a> {
    pub project_dir: &'a Path,
    pub turn_id: &'a str,
    pub episode: &'a str,
    pub start_token: usize,
    pub end_token: usize,
    pub script_text: &'a str,
    pub heard_whisper: &'a str,
    pub heard_phonon: Option<&'a str>,
}

/// Build argv for `scriptcast-review add` (program resolved separately).
pub fn add_args(request: &AddRequest<'_>, form: &DecisionForm) -> Vec<String> {
    let mut args = vec![
        "add".into(),
        request.project_dir.display().to_string(),
        "--turn-id".into(),
        request.turn_id.into(),
        "--episode".into(),
        request.episode.into(),
        "--start-token".into(),
        request.start_token.to_string(),
        "--end-token".into(),
        request.end_token.to_string(),
        "--script-text".into(),
        request.script_text.into(),
        "--heard-whisper".into(),
        request.heard_whisper.into(),
        "--outcome".into(),
        form.outcome.as_str().into(),
    ];
    if let Some(phonon) = request.heard_phonon.filter(|s| !s.is_empty()) {
        args.extend(["--heard-phonon".into(), phonon.into()]);
    }
    if !form.correction.trim().is_empty() {
        args.extend(["--correction".into(), form.correction.clone()]);
    }
    if form.outcome == Outcome::RecognizerMisheard {
        for pair in form.accepts.iter().filter(|p| p.checked) {
            args.extend(["--accept".into(), format!("{}={}", pair.word, pair.heard)]);
        }
    }
    args
}

pub const SAVED_BANNER: &str = "Saved · applies from the next Verify";
pub const NO_RETRAIN: &str =
    "Saved locally. Used by the next verification; does not retrain any model.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_word_diff_enables_one_accept() {
        let pairs = replace_pairs("the potestas clause", "the potty clause");
        assert_eq!(pairs, vec![("potestas".into(), "potty".into())]);
        let form = DecisionForm::for_turn("t1", "the potestas clause", "the potty clause");
        assert!(form.single_accept().is_some());
        assert_eq!(form.single_accept().unwrap().word, "potestas");
        assert!(form.single_accept().unwrap().checked);
    }

    #[test]
    fn multi_pair_lists_each_checkbox() {
        let pairs = replace_pairs("alpha beta gamma", "alfa better gamma");
        assert_eq!(
            pairs,
            vec![
                ("alpha".into(), "alfa".into()),
                ("beta".into(), "better".into()),
            ]
        );
        let form = DecisionForm::for_turn("t1", "alpha beta gamma", "alfa better gamma");
        assert!(form.single_accept().is_none());
        assert_eq!(form.accepts.len(), 2);
        // Several pairs from one span start unticked: the operator chooses each.
        assert!(form.accepts.iter().all(|p| !p.checked));
    }

    #[test]
    fn calibrate_hint_only_for_audio_wrong_single_word() {
        let mut form = DecisionForm::for_turn("t1", "record", "wreckerd");
        form.outcome = Outcome::AudioWrong;
        assert!(
            form.calibrate_hint()
                .unwrap()
                .contains("scriptcast-calibrate record record")
        );
        form.outcome = Outcome::RecognizerMisheard;
        assert!(form.calibrate_hint().is_none());
        let mut multi = DecisionForm::for_turn("t1", "a b", "x y");
        multi.outcome = Outcome::AudioWrong;
        assert!(multi.calibrate_hint().is_none());
    }

    #[test]
    fn choose_outcome_updates_form_for_both_views() {
        let mut form = DecisionForm::for_turn("t1", "a", "b");
        form.saved = true;
        choose_outcome(&mut form, Outcome::AudioWrong);
        assert_eq!(form.outcome, Outcome::AudioWrong);
        assert!(!form.saved);
        choose_outcome(&mut form, Outcome::ScriptNeedsCorrection);
        assert_eq!(form.outcome, Outcome::ScriptNeedsCorrection);
        assert_eq!(form.outcome.card_title(), "Correct script only");
    }

    #[test]
    fn add_args_includes_accepts_only_for_misheard() {
        let mut form = DecisionForm::for_turn("t1", "potestas", "potty");
        form.outcome = Outcome::RecognizerMisheard;
        let request = AddRequest {
            project_dir: Path::new("/tmp/proj"),
            turn_id: "t1",
            episode: "EP03",
            start_token: 0,
            end_token: 1,
            script_text: "potestas",
            heard_whisper: "potty",
            heard_phonon: Some("potestas"),
        };
        let args = add_args(&request, &form);
        assert!(args.contains(&"--accept".into()));
        assert!(args.contains(&"potestas=potty".into()));
        form.outcome = Outcome::AudioWrong;
        let request = AddRequest {
            heard_phonon: None,
            ..request
        };
        let args = add_args(&request, &form);
        assert!(!args.iter().any(|a| a == "--accept"));
    }

    #[test]
    fn common_or_multi_pair_accepts_start_unticked() {
        let form = DecisionForm::for_turn("t", "trained in the loop", "trained it the loop");
        assert_eq!(form.accepts.len(), 1);
        assert!(form.accepts[0].risky);
        assert!(!form.accepts[0].checked);
        let form = DecisionForm::for_turn("t", "alpha beta gamma", "alfa better gamma");
        assert!(form.accepts.iter().all(|p| !p.checked));
        let form = DecisionForm::for_turn("t", "the potestas clause", "the potty clause");
        assert!(form.accepts[0].checked && !form.accepts[0].risky);
    }
}
