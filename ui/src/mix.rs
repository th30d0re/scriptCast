//! `mix.yaml` model and deterministic, local validation.
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub const OVERLAY_GAIN_MIN: f32 = -40.0;
pub const OVERLAY_GAIN_MAX: f32 = 0.0;
pub const OVERLAY_GAIN_DEFAULT: f32 = -22.0;
pub const TURN_GAIN_MIN: f32 = -12.0;
pub const TURN_GAIN_MAX: f32 = 12.0;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mix {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlays: Vec<Overlay>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gains: Vec<Gain>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overlay {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_equals: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_contains: Option<String>,
    pub after_word: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub delay_ms: i64,
    #[serde(default = "default_overlay_gain")]
    pub gain_db: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gain {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_equals: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_contains: Option<String>,
    pub gain_db: f32,
}

fn is_zero(value: &i64) -> bool {
    *value == 0
}
fn default_overlay_gain() -> f32 {
    OVERLAY_GAIN_DEFAULT
}

impl Overlay {
    pub fn clamp_gain(&mut self) {
        self.gain_db = self.gain_db.clamp(OVERLAY_GAIN_MIN, OVERLAY_GAIN_MAX);
    }
    pub fn selector(&self) -> Option<&str> {
        self.turn_equals
            .as_deref()
            .or(self.turn_contains.as_deref())
    }
}
impl Gain {
    pub fn clamp_gain(&mut self) {
        self.gain_db = self.gain_db.clamp(TURN_GAIN_MIN, TURN_GAIN_MAX);
    }
    pub fn selector(&self) -> Option<&str> {
        self.turn_equals
            .as_deref()
            .or(self.turn_contains.as_deref())
    }
}
impl Mix {
    pub fn clamp_gains(&mut self) {
        self.overlays.iter_mut().for_each(Overlay::clamp_gain);
        self.gains.iter_mut().for_each(Gain::clamp_gain);
    }
}

pub fn load(path: &Path) -> Result<Mix, String> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Mix::default()),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let mut mix = parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    mix.clamp_gains();
    Ok(mix)
}

pub fn save(path: &Path, mix: &Mix) -> Result<(), String> {
    let mut safe = mix.clone();
    safe.clamp_gains();
    let text = if safe == Mix::default() {
        "overlays: []\ngains: []\n".to_owned()
    } else {
        format_yaml(&safe)
    };
    fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Overlays,
    Gains,
}

fn strip_comment(value: &str) -> &str {
    let mut single = false;
    let mut double = false;
    for (i, c) in value.char_indices() {
        match c {
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '#' if !single && !double => return value[..i].trim_end(),
            _ => {}
        }
    }
    value.trim_end()
}
fn scalar(value: &str) -> Result<String, String> {
    let value = strip_comment(value).trim();
    if value.starts_with('"') {
        serde_json::from_str(value).map_err(|e| format!("invalid string: {e}"))
    } else if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
        Ok(value[1..value.len() - 1].replace("''", "'"))
    } else {
        Ok(value.to_owned())
    }
}
fn finish(
    section: Section,
    fields: &mut BTreeMap<String, String>,
    mix: &mut Mix,
) -> Result<(), String> {
    if fields.is_empty() {
        return Ok(());
    }
    let take = |fields: &mut BTreeMap<String, String>, key: &str| {
        fields.remove(key).ok_or_else(|| format!("missing {key}"))
    };
    let opt = |fields: &mut BTreeMap<String, String>, key: &str| fields.remove(key);
    match section {
        Section::Overlays => mix.overlays.push(Overlay {
            file: take(fields, "file")?,
            turn_equals: opt(fields, "turn_equals"),
            turn_contains: opt(fields, "turn_contains"),
            after_word: take(fields, "after_word")?,
            delay_ms: opt(fields, "delay_ms")
                .map(|v| v.parse())
                .transpose()
                .map_err(|_| "invalid delay_ms")?
                .unwrap_or(0),
            gain_db: opt(fields, "gain_db")
                .map(|v| v.parse())
                .transpose()
                .map_err(|_| "invalid gain_db")?
                .unwrap_or(OVERLAY_GAIN_DEFAULT),
        }),
        Section::Gains => mix.gains.push(Gain {
            turn_equals: opt(fields, "turn_equals"),
            turn_contains: opt(fields, "turn_contains"),
            gain_db: take(fields, "gain_db")?
                .parse()
                .map_err(|_| "invalid gain_db")?,
        }),
    }
    let selector_count = match section {
        Section::Overlays => {
            let rule = mix.overlays.last().unwrap();
            usize::from(rule.turn_equals.is_some()) + usize::from(rule.turn_contains.is_some())
        }
        Section::Gains => {
            let rule = mix.gains.last().unwrap();
            usize::from(rule.turn_equals.is_some()) + usize::from(rule.turn_contains.is_some())
        }
    };
    if selector_count != 1 {
        return Err("mix rule needs exactly one of turn_equals or turn_contains".into());
    }
    if !fields.is_empty() {
        return Err(format!("unknown field {}", fields.keys().next().unwrap()));
    }
    Ok(())
}
fn parse(text: &str) -> Result<Mix, String> {
    let mut mix = Mix::default();
    let mut section = None;
    let mut fields = BTreeMap::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "overlays:" || line == "overlays: []" || line == "gains:" || line == "gains: []"
        {
            if let Some(s) = section {
                finish(s, &mut fields, &mut mix)?;
            }
            section = Some(if line.starts_with("overlays") {
                Section::Overlays
            } else {
                Section::Gains
            });
            continue;
        }
        let Some(s) = section else {
            return Err("expected overlays: or gains:".into());
        };
        let item = line.strip_prefix("- ");
        if item.is_some() {
            finish(s, &mut fields, &mut mix)?;
        }
        let pair = item.unwrap_or(line);
        let (key, value) = pair
            .split_once(':')
            .ok_or_else(|| format!("invalid line {line:?}"))?;
        fields.insert(key.trim().to_owned(), scalar(value)?);
    }
    if let Some(s) = section {
        finish(s, &mut fields, &mut mix)?;
    }
    Ok(mix)
}
fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".into())
}
fn selector(out: &mut String, equals: &Option<String>, contains: &Option<String>) {
    if let Some(v) = equals {
        out.push_str(&format!("    turn_equals: {}\n", quoted(v)));
    }
    if let Some(v) = contains {
        out.push_str(&format!("    turn_contains: {}\n", quoted(v)));
    }
}
fn format_yaml(mix: &Mix) -> String {
    let mut out = String::new();
    if mix.overlays.is_empty() {
        out.push_str("overlays: []\n")
    } else {
        out.push_str("overlays:\n");
        for o in &mix.overlays {
            out.push_str(&format!("  - file: {}\n", quoted(&o.file)));
            selector(&mut out, &o.turn_equals, &o.turn_contains);
            out.push_str(&format!(
                "    after_word: {}\n    delay_ms: {}\n    gain_db: {}\n",
                quoted(&o.after_word),
                o.delay_ms,
                o.gain_db
            ));
        }
    }
    if mix.gains.is_empty() {
        out.push_str("gains: []\n")
    } else {
        out.push_str("gains:\n");
        for g in &mix.gains {
            out.push_str("  - ");
            if let Some(v) = &g.turn_equals {
                out.push_str(&format!("turn_equals: {}\n", quoted(v)))
            } else if let Some(v) = &g.turn_contains {
                out.push_str(&format!("turn_contains: {}\n", quoted(v)))
            }
            out.push_str(&format!("    gain_db: {}\n", g.gain_db));
        }
    }
    out
}

pub fn normalize(word: &str) -> String {
    word.chars()
        .flat_map(char::to_lowercase)
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

/// Returns the tool's exact failure sentence when no normalized script word matches.
pub fn unmatched_word_error(
    overlay: &Overlay,
    turn_index: usize,
    turn_text: &str,
) -> Option<String> {
    let pattern = Regex::new(&overlay.after_word).ok()?;
    turn_text
        .split_whitespace()
        .any(|word| pattern.is_match(&normalize(word)))
        .then_some(())
        .is_none()
        .then(|| {
            format!(
                "no recognised word matches '{}' in turn {turn_index}",
                overlay.after_word
            )
        })
}

pub fn config_path(project_root: &Path) -> PathBuf {
    project_root.join("mix.yaml")
}
pub fn asr_path(episode_dir: &Path, episode: &str) -> PathBuf {
    episode_dir.join(format!("{episode}.captions.json.asr.json"))
}
pub fn mixed_path(episode_dir: &Path, episode: &str) -> PathBuf {
    episode_dir.join(format!("{episode}_mixed.mp3"))
}
