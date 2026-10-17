//! Clip maker, reference maker, audition, calibrate, outliers, and stress check.
use crate::Browser;
use eframe::egui::{self, Ui};
use crate::{phase_m::*, project::Report, theme as t};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
    time::SystemTime,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ToolKind {
    Words,
    Register,
    Candidates,
    WriteReference,
    Audition,
    Calibrate,
    Outliers,
    Stress,
}

impl ToolKind {
    fn preset(self) -> &'static str {
        match self {
            Self::Words => "clip-words",
            Self::Register => "clip-register",
            Self::Candidates => "reference-candidates",
            Self::WriteReference => "reference-write",
            Self::Audition => "audition",
            Self::Calibrate => "calibrate",
            Self::Outliers => "outliers",
            Self::Stress => "stress",
        }
    }
}

struct ToolRun {
    kind: ToolKind,
    path: PathBuf,
    cancelled: bool,
}

pub struct PhaseMUi {
    pub clip: bool,
    pub reference: bool,
    pub audition: bool,
    pub calibrate: bool,
    pub outliers: bool,
    pub stress: bool,
    pub ref_source: String,
    pub audition_speaker: String,
    source: String,
    words: Option<WordsJson>,
    span: Option<WordSpan>,
    clip_id: String,
    speaker: String,
    citation: String,
    url: String,
    note: String,
    pad_ms: u32,
    registered: Option<ClipRegistered>,
    words_cache: HashMap<(PathBuf, SystemTime), WordsJson>,
    seconds: f64,
    candidates: Vec<CandidateRun>,
    expanded: Option<usize>,
    ref_start: String,
    ref_end: String,
    transcript: String,
    ref_name: String,
    peak_target: String,
    written: Option<ReferenceWritten>,
    chips: Vec<String>,
    audition_result: Option<AuditionJson>,
    word: String,
    reading: String,
    spellings: String,
    cal_speaker: String,
    calibration: Option<CalibrateJson>,
    outlier_report: Option<OutliersJson>,
    outlier_row: Option<usize>,
    stress_report: Option<StressJson>,
    keep_audition: Option<PathBuf>,
    keep_calibrate: Option<PathBuf>,
    job: Option<ToolRun>,
    notice: Option<String>,
    progress: String,
    footer_status: String,
    footer_primary: String,
    footer_enabled: bool,
    wave_path: Option<PathBuf>,
    wave_origin: f64,
    wave_rx: Option<Receiver<Result<(crate::waveform::Envelope, f64), String>>>,
    wave: Option<Result<crate::waveform::Envelope, String>>,
}

impl Default for PhaseMUi {
    fn default() -> Self {
        Self {
            pad_ms: 100,
            seconds: 9.0,
            peak_target: "0.92".into(),
            reading: "default".into(),
            ..Self::default_rest()
        }
    }
}

impl PhaseMUi {
    fn default_rest() -> Self {
        Self {
            clip: false,
            reference: false,
            audition: false,
            calibrate: false,
            outliers: false,
            stress: false,
            source: String::new(),
            words: None,
            span: None,
            clip_id: String::new(),
            speaker: String::new(),
            citation: String::new(),
            url: String::new(),
            note: String::new(),
            pad_ms: 0,
            registered: None,
            words_cache: HashMap::new(),
            ref_source: String::new(),
            seconds: 0.0,
            candidates: Vec::new(),
            expanded: None,
            ref_start: String::new(),
            ref_end: String::new(),
            transcript: String::new(),
            ref_name: String::new(),
            peak_target: String::new(),
            written: None,
            audition_speaker: String::new(),
            chips: Vec::new(),
            audition_result: None,
            word: String::new(),
            reading: String::new(),
            spellings: String::new(),
            cal_speaker: String::new(),
            calibration: None,
            outlier_report: None,
            outlier_row: None,
            stress_report: None,
            keep_audition: None,
            keep_calibrate: None,
            job: None,
            notice: None,
            progress: String::new(),
            footer_status: String::new(),
            footer_primary: String::new(),
            footer_enabled: false,
            wave_path: None,
            wave_origin: 0.0,
            wave_rx: None,
            wave: None,
        }
    }
}

struct Person {
    id: String,
    name: String,
    engine: String,
}

impl Browser {
    pub(crate) fn phase_m_windows(&mut self, ctx: &egui::Context) {
        self.clip_window_body(ctx);
        self.stress_window(ctx);
    }

    pub(crate) fn phase_m_replaces(&self, tab: &str) -> bool {
        match tab {
            "voices" => self.makers.reference || self.makers.audition,
            "pronunciations" => self.makers.calibrate || self.makers.outliers,
            _ => false,
        }
    }

    pub(crate) fn phase_m_fill(&mut self, ui: &mut Ui, report: &Report) {
        if self.makers.audition {
            self.audition_body(ui, report);
        } else if self.makers.reference {
            self.reference_body(ui);
        } else {
            self.m4_body(ui, report);
        }
    }

    pub(crate) fn phase_m_footer(&mut self, ui: &mut Ui) {
        let primary = self.makers.footer_primary.clone();
        let status = self.makers.footer_status.clone();
        let enabled = self.makers.footer_enabled;
        let (cancel, go) = crate::phase_l::tool_footer(ui, &status, &primary, enabled);
        if cancel {
            self.makers.reference = false;
            self.makers.audition = false;
            self.makers.calibrate = false;
            self.makers.outliers = false;
        }
        if go && self.makers.audition {
            let index = self
                .makers
                .audition_result
                .as_ref()
                .map(|result| result.candidates.iter().map(|c| c.mean).collect::<Vec<_>>())
                .as_deref()
                .and_then(winner_index);
            if let Some(index) = index {
                self.use_audition_reference(index);
            }
        } else if go && self.makers.reference {
            self.begin_reference_write();
        } else if go && self.makers.calibrate {
            self.begin_calibrate(true);
        }
    }

    pub(crate) fn poll_phase_m(&mut self, ctx: &egui::Context) {
        if let Some(result) = self
            .makers
            .wave_rx
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.makers.wave_rx = None;
            match result {
                Ok((envelope, origin)) => {
                    self.makers.wave_origin = origin;
                    self.makers.wave = Some(Ok(envelope));
                }
                Err(error) => self.makers.wave = Some(Err(error)),
            }
        }
        let Some(job) = self.makers.job.as_ref() else {
            return;
        };
        let path = job.path.clone();
        let cancelled = job.cancelled;
        let kind = job.kind;
        let Ok(record) = crate::detached::read(&path) else {
            return;
        };
        self.makers.progress = progress_line(&tail_log(&record.log));
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
        let finished = record.exit.is_some()
            || (record.pid != 0 && !crate::detached::alive(record.pid));
        if !finished {
            return;
        }
        self.makers.job = None;
        if cancelled {
            self.makers.notice = Some("Cancelled".into());
            self.makers.progress.clear();
            return;
        }
        let log = tail_log(&record.log);
        let parsed = last_json_line(&log)
            .ok_or_else(|| "Tool finished without JSON".to_owned())
            .and_then(|line| apply_tool_json(kind, line.as_bytes()));
        match parsed {
            Ok(outcome) => self.store_outcome(outcome),
            Err(error) => self.makers.notice = Some(error),
        }
        self.makers.progress.clear();
    }

    pub(crate) fn cancel_phase_m(&mut self) {
        if let Some(job) = self.makers.job.as_mut() {
            job.cancelled = true;
            if let Ok(record) = crate::detached::read(&job.path) {
                crate::detached::stop(&record);
            }
        }
    }

    pub(crate) fn apply_phase_m_cli(&mut self, cli: &crate::capture::Cli) {
        if let Some(source) = &cli.clip_source {
            self.makers.source = source.display().to_string();
        }
        if cli.clip_maker {
            self.makers.clip = true;
            self.project.tab = "clips".into();
            self.project.data_open = true;
        }
        if let Some(path) = &cli.words_fixture {
            self.load_words_fixture(path);
        }
        if cli.reference_maker {
            self.makers.reference = true;
            self.project.tab = "voices".into();
            self.project.data_open = true;
        }
        if let Some(path) = &cli.candidates_fixture {
            self.load_candidates_fixture(path);
        }
        if let Some(path) = &cli.audition_fixture {
            self.load_audition_fixture(path);
        }
        if let Some(path) = &cli.calibrate_fixture {
            self.load_calibrate_fixture(path);
        }
        if let Some(path) = &cli.outliers_fixture {
            self.load_outliers_fixture(path);
        }
    }

    pub(crate) fn start_stress_check(&mut self, index: usize) {
        self.makers.stress = true;
        self.makers.stress_report = None;
        let text = self
            .editing
            .turns
            .as_ref()
            .and_then(|turns| turns.turns.get(index))
            .map(|turn| turn.body.clone())
            .filter(|body| !body.trim().is_empty())
            .or_else(|| {
                self.episode
                    .as_ref()
                    .and_then(|episode| episode.turns.get(index))
                    .map(|turn| turn.first_words.clone())
            });
        let wav = self
            .episode
            .as_ref()
            .and_then(|episode| episode.turns.get(index))
            .and_then(|turn| turn.wav_paths.first())
            .cloned();
        match (wav, text) {
            (Some(wav), Some(text)) => {
                let call = stress_invocation(&wav.display().to_string(), &text);
                self.launch_tool(ToolKind::Stress, call);
            }
            _ => {
                self.makers.notice =
                    Some("Check stress needs the turn text and a rendered wav.".into());
            }
        }
    }

    pub(crate) fn open_outliers(&mut self) {
        self.makers.outliers = true;
        self.project.data_open = true;
        self.project.tab = "pronunciations".into();
        if self.makers.outlier_report.is_some() || self.makers.job.is_some() {
            return;
        }
        let Some(episode) = &self.episode else {
            self.makers.notice = Some("Open an episode to find outliers.".into());
            return;
        };
        let Some(transcript) = episode.script_path.clone() else {
            self.makers.notice = Some("This episode has no transcript path.".into());
            return;
        };
        let call = outliers_invocation(
            &episode.dir.display().to_string(),
            &transcript.display().to_string(),
        );
        self.launch_tool(ToolKind::Outliers, call);
    }

    fn load_words_fixture(&mut self, path: &Path) {
        match read_fixture::<WordsJson>(path) {
            Ok(words) => {
                if self.makers.source.is_empty() {
                    self.makers.source = words.source.clone();
                }
                if !words.words.is_empty() {
                    self.makers.span = Some(WordSpan {
                        anchor: 0,
                        focus: words.words.len() - 1,
                    });
                }
                self.makers.words = Some(words);
                self.makers.clip = true;
                self.project.tab = "clips".into();
                self.project.data_open = true;
            }
            Err(error) => self.makers.notice = Some(error),
        }
    }

    fn load_candidates_fixture(&mut self, path: &Path) {
        match read_fixture::<CandidatesJson>(path) {
            Ok(parsed) => {
                self.makers.ref_source = parsed.source.clone();
                if let Some(first) = parsed.candidates.first() {
                    self.makers.expanded = Some(0);
                    self.makers.ref_start = seconds_arg(first.start);
                    self.makers.ref_end = seconds_arg(first.end);
                    self.makers.transcript = first.text.clone();
                }
                self.makers.candidates = parsed.candidates;
                self.makers.reference = true;
                self.project.tab = "voices".into();
                self.project.data_open = true;
            }
            Err(error) => self.makers.notice = Some(error),
        }
    }

    fn load_audition_fixture(&mut self, path: &Path) {
        match read_fixture::<AuditionJson>(path) {
            Ok(parsed) => {
                self.makers.audition_speaker = parsed.speaker.clone();
                self.makers.chips = parsed
                    .candidates
                    .iter()
                    .map(|candidate| candidate.reference.clone())
                    .collect();
                self.makers.audition_result = Some(parsed);
                self.makers.audition = true;
                self.project.tab = "voices".into();
                self.project.data_open = true;
            }
            Err(error) => self.makers.notice = Some(error),
        }
    }

    fn load_calibrate_fixture(&mut self, path: &Path) {
        match read_fixture::<CalibrateJson>(path) {
            Ok(parsed) => {
                self.makers.word = parsed.word.clone();
                self.makers.reading = parsed.reading.clone();
                self.makers.calibration = Some(parsed);
                self.makers.calibrate = true;
                self.project.tab = "pronunciations".into();
                self.project.data_open = true;
            }
            Err(error) => self.makers.notice = Some(error),
        }
    }

    fn load_outliers_fixture(&mut self, path: &Path) {
        match read_fixture::<OutliersJson>(path) {
            Ok(parsed) => {
                self.makers.outlier_row = (!parsed.outliers.is_empty()).then_some(0);
                self.makers.outlier_report = Some(parsed);
                self.makers.outliers = true;
                if self.project.tab.is_empty() {
                    self.project.tab = "pronunciations".into();
                }
                self.project.data_open = true;
            }
            Err(error) => self.makers.notice = Some(error),
        }
    }

    fn store_outcome(&mut self, outcome: Outcome) {
        self.makers.notice = None;
        match outcome {
            Outcome::Words(words) => {
                if let Ok(key) = source_cache_key(Path::new(&self.makers.source)) {
                    self.makers.words_cache.insert(key, words.clone());
                }
                if !words.words.is_empty() {
                    self.makers.span = Some(WordSpan {
                        anchor: 0,
                        focus: 0,
                    });
                }
                self.makers.words = Some(words);
            }
            Outcome::Registered(registered) => self.makers.registered = Some(registered),
            Outcome::Candidates(parsed) => {
                if let Some(first) = parsed.candidates.first() {
                    self.makers.expanded = Some(0);
                    self.makers.ref_start = seconds_arg(first.start);
                    self.makers.ref_end = seconds_arg(first.end);
                    self.makers.transcript = first.text.clone();
                }
                self.makers.candidates = parsed.candidates;
            }
            Outcome::Written(written) => self.makers.written = Some(written),
            Outcome::Audition(parsed) => self.makers.audition_result = Some(parsed),
            Outcome::Calibration(parsed) => {
                if parsed.written {
                    self.project.stale = true;
                }
                self.makers.calibration = Some(parsed);
            }
            Outcome::Outliers(parsed) => {
                self.makers.outlier_row = (!parsed.outliers.is_empty()).then_some(0);
                self.makers.outlier_report = Some(parsed);
            }
            Outcome::Stress(parsed) => self.makers.stress_report = Some(parsed),
        }
    }

    fn launch_tool(&mut self, kind: ToolKind, call: Invocation) {
        if self.makers.job.is_some() {
            self.makers.notice = Some("A tool is already running.".into());
            return;
        }
        let root = self
            .project
            .root
            .clone()
            .or_else(|| self.episode.as_ref().and_then(|e| e.project_root.clone()))
            .unwrap_or_else(std::env::temp_dir);
        let resolved = match crate::cli::resolve(
            call.command,
            Some(&root),
            self.preferences.cli_dir.as_deref(),
        ) {
            Ok(resolved) => resolved,
            Err(error) => {
                self.makers.notice = Some(error);
                return;
            }
        };
        let mut args = resolved.prefix.clone();
        args.extend(call.args.iter().map(std::ffi::OsString::from));
        let spec = crate::runner::JobSpec {
            program: resolved.program.clone(),
            args,
            cwd: root.clone(),
            synthesizes: false,
            overwrites: vec![],
            resolved,
        };
        let plan = crate::detached::CommandPlan::new(&spec);
        match crate::detached::launch(&root, kind.preset(), vec![plan], None) {
            Ok((path, _)) => {
                self.makers.notice = None;
                self.makers.progress = "Running".into();
                self.makers.job = Some(ToolRun {
                    kind,
                    path,
                    cancelled: false,
                });
            }
            Err(error) => self.makers.notice = Some(error),
        }
    }

    fn job_line(&mut self, ui: &mut Ui, kind: ToolKind) {
        let running = self.makers.job.as_ref().is_some_and(|job| job.kind == kind);
        if running {
            ui.label(t::text(
                format!("Running · {}", self.makers.progress),
                t::CAPTION,
                t::TEXT_MUTED,
            ));
            if ui.button("Cancel").clicked() {
                self.cancel_phase_m();
            }
        }
        if let Some(notice) = &self.makers.notice {
            ui.label(t::text(notice, t::CAPTION, t::STATUS_FAILED));
        }
    }

    fn clip_window_body(&mut self, ctx: &egui::Context) {
        if !self.makers.clip {
            return;
        }
        let mut transcribe = false;
        let mut register = false;
        let mut play = false;
        let mut copy = false;
        let mut insert = false;
        let mut cancel = false;
        let mut picked: Option<PathBuf> = None;
        let step = if self.makers.words.is_none() {
            1
        } else if self.makers.span.is_none() {
            2
        } else {
            3
        };
        egui::Modal::new(egui::Id::new("clip_maker"))
            .backdrop_color(t::BG_SCRIM)
            .frame(
                egui::Frame::new()
                    .fill(t::BG_RAISED)
                    .stroke(egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT))
                    .corner_radius(t::RADIUS_DIALOG as u8)
                    .inner_margin(t::SPACE_16),
            )
            .show(ctx, |ui| {
                ui.set_width(t::CLIP_MAKER_WIDTH);
                ui.set_min_height(t::CLIP_MAKER_HEIGHT);
                ui.label(t::heading("Clip maker", t::DIALOG_TITLE));
                step_row(ui, step);
                crate::phase_l::two_columns(ui, t::CLIP_MAKER_SPLIT, |columns| {
                    self.clip_source_column(
                        &mut columns[0],
                        &mut picked,
                        &mut transcribe,
                        &mut play,
                    );
                    self.clip_details(&mut columns[1], &mut register, &mut cancel);
                });
                if self.makers.registered.is_some() {
                    self.register_card(ui, &mut copy, &mut insert);
                }
            });
        if let Some(path) = picked {
            self.makers.source = path.display().to_string();
            self.makers.words = None;
            self.makers.span = None;
            self.makers.registered = None;
        }
        if transcribe {
            self.begin_transcribe();
        }
        if play {
            self.play_clip_range();
        }
        if register {
            self.begin_register();
        }
        if copy && let Some(turn) = self.makers.registered.as_ref().map(|r| r.turn.clone()) {
            ctx.copy_text(turn);
        }
        if cancel {
            self.makers.clip = false;
        }
        if insert {
            self.insert_registered_clip();
        }
    }

    fn clip_source_column(
        &mut self,
        ui: &mut Ui,
        picked: &mut Option<PathBuf>,
        transcribe: &mut bool,
        play: &mut bool,
    ) {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.makers.source)
                    .desired_width(ui.available_width() - t::TEXT_WIDTH_CAP)
                    .hint_text("Source audio or video"),
            );
            if ui.button("Choose…").clicked() {
                *picked = rfd::FileDialog::new().pick_file();
            }
            let busy = self.makers.job.is_some();
            *transcribe = t::primary_button(ui, "Transcribe", None, !busy)
                .on_disabled_hover_text("A tool is already running")
                .clicked();
        });
        self.job_line(ui, ToolKind::Words);
        self.word_picker(ui, play);
    }

    fn word_picker(&mut self, ui: &mut Ui, play: &mut bool) {
        let Some(words) = self.makers.words.clone() else {
            ui.label(t::text(
                "Transcribe the source to pick words.",
                t::CAPTION,
                t::TEXT_MUTED,
            ));
            return;
        };
        let path = PathBuf::from(&self.makers.source);
        if path.is_file() {
            self.ensure_wave(&path, words.duration, ui.ctx());
        }
        let (start, end) = self
            .makers
            .span
            .and_then(|span| span_times(&words.words, span))
            .unwrap_or((0.0, words.duration));
        self.paint_wave(ui, words.duration, start, end, true);
        if let Some(span) = self.makers.span
            && let Some((start, end)) = span_times(&words.words, span)
        {
            ui.horizontal(|ui| {
                ui.label(t::text(
                    format!(
                        "{} → {} · {:.1} s",
                        clock(start),
                        clock(end),
                        (end - start).max(0.0)
                    ),
                    t::BODY,
                    t::TEXT_PRIMARY,
                ));
                *play = ui.button("Play range").clicked();
            });
        }
        if let Some(next) = flowing_words(ui, &words.words, self.makers.span) {
            self.makers.span = Some(next);
        }
    }

    fn clip_details(&mut self, ui: &mut Ui, register: &mut bool, cancel: &mut bool) {
        let collision = {
            let ids = self.clip_id_list();
            let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
            id_collides(&refs, &self.makers.clip_id)
        };
        ui.label(t::heading("Details", t::SECTION_TITLE));
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(t::text("Id", t::CAPTION, t::TEXT_MUTED));
                ui.add(
                    egui::TextEdit::singleline(&mut self.makers.clip_id)
                        .desired_width(t::TEXT_WIDTH_CAP),
                );
            });
            ui.vertical(|ui| {
                ui.label(t::text("Pad ms", t::CAPTION, t::TEXT_MUTED));
                ui.add(egui::DragValue::new(&mut self.makers.pad_ms).range(0..=5_000));
            });
        });
        if collision {
            ui.label(t::text("Id already exists", t::CAPTION, t::STATUS_FAILED));
        }
        let archives = self.people(|person| person.engine == "archive");
        ui.label(t::text(
            "Speaker · archive speakers only",
            t::CAPTION,
            t::TEXT_MUTED,
        ));
        if archives.is_empty() {
            ui.label(t::text(
                "No archive speakers in voices.",
                t::CAPTION,
                t::TEXT_MUTED,
            ));
        } else {
            let selected = archives
                .iter()
                .find(|person| person.id == self.makers.speaker)
                .map(|person| person.name.clone())
                .unwrap_or_else(|| "Choose speaker".into());
            egui::ComboBox::from_id_salt("clip_archive_speaker")
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for person in &archives {
                        ui.selectable_value(
                            &mut self.makers.speaker,
                            person.id.clone(),
                            &person.name,
                        );
                    }
                });
        }
        labeled(ui, "Citation", &mut self.makers.citation);
        labeled(ui, "Origin URL", &mut self.makers.url);
        ui.label(t::text("Content note", t::CAPTION, t::TEXT_MUTED));
        ui.text_edit_multiline(&mut self.makers.note);
        let ready = !collision
            && !self.makers.clip_id.trim().is_empty()
            && self.makers.span.is_some()
            && self.makers.words.is_some()
            && self.makers.job.is_none();
        let reason = if collision {
            "Disabled: duplicate id"
        } else if self.makers.span.is_none() {
            "Select a word range first"
        } else if self.makers.job.is_some() {
            "A tool is already running"
        } else {
            "Enter an id"
        };
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Max), |ui| {
            ui.horizontal(|ui| {
                *register = t::primary_button(ui, "Register clip", None, ready)
                    .on_disabled_hover_text(reason)
                    .clicked();
                *cancel = t::secondary_button(ui, "Cancel", None, true).clicked();
            });
        });
    }

    fn register_card(&mut self, ui: &mut Ui, copy: &mut bool, insert: &mut bool) {
        let Some(registered) = self.makers.registered.clone() else {
            return;
        };
        ui.separator();
        ui.label(t::text(
            format!("Registered {}", registered.id),
            t::BODY,
            t::TEXT_PRIMARY,
        ));
        ui.label(
            egui::RichText::new(&registered.turn)
                .font(t::mono(t::BODY))
                .color(t::TEXT_PRIMARY),
        );
        let after = self.selected_turn_label();
        let can_insert = self.selected.is_some() && !self.makers.speaker.is_empty();
        ui.horizontal(|ui| {
            *copy = t::secondary_button(ui, "Copy", None, true).clicked();
            let label = format!("Insert into script after {after}");
            *insert = t::primary_button(ui, &label, None, can_insert)
                .on_disabled_hover_text("Select a turn and an archive speaker")
                .clicked();
        });
    }

    fn begin_transcribe(&mut self) {
        let source = self.makers.source.trim().to_owned();
        if source.is_empty() {
            self.makers.notice = Some("Choose a source file.".into());
            return;
        }
        let path = PathBuf::from(&source);
        if let Ok(key) = source_cache_key(&path)
            && let Some(cached) = self.makers.words_cache.get(&key).cloned()
        {
            self.makers.words = Some(cached);
            self.makers.notice = None;
            return;
        }
        self.launch_tool(ToolKind::Words, clip_words_invocation(&source));
    }

    fn begin_register(&mut self) {
        let Some(words) = self.makers.words.clone() else {
            return;
        };
        let Some(span) = self.makers.span else {
            return;
        };
        let Some((start, end)) = span_times(&words.words, span) else {
            return;
        };
        let speaker = self
            .people(|person| person.id == self.makers.speaker)
            .into_iter()
            .next()
            .map(|person| person.name)
            .unwrap_or_default();
        let call = clip_register_invocation(&ClipRegister {
            source: self.makers.source.trim().to_owned(),
            id: self.makers.clip_id.clone(),
            start,
            end,
            speaker,
            citation: self.makers.citation.clone(),
            url: self.makers.url.clone(),
            content_note: self.makers.note.clone(),
            pad_ms: self.makers.pad_ms,
        });
        self.makers.registered = None;
        self.launch_tool(ToolKind::Register, call);
    }

    fn play_clip_range(&mut self) {
        let Some(words) = self.makers.words.clone() else {
            return;
        };
        let Some(span) = self.makers.span else {
            return;
        };
        let Some((start, end)) = span_times(&words.words, span) else {
            return;
        };
        let source = self.makers.source.clone();
        self.audition_project_media(Path::new(&source), start, Some(end));
    }

    fn insert_registered_clip(&mut self) {
        let Some(registered) = self.makers.registered.clone() else {
            return;
        };
        let Some(selected) = self.selected else {
            return;
        };
        let position = selected + 1;
        self.editing.k.action = Some(crate::phase_k::MenuAction::Apply(
            crate::structure::EditOp::Insert {
                position,
                speaker_id: self.makers.speaker.clone(),
                timestamp: Some("00:00".into()),
                body: clip_insert_body(&registered.turn),
            },
            position,
        ));
    }

    fn reference_body(&mut self, ui: &mut Ui) {
        let mut find = false;
        let mut play_row: Option<usize> = None;
        let mut play_span = false;
        let mut reveal = false;
        let mut use_ref = false;
        let mut picked: Option<PathBuf> = None;
        ui.horizontal(|ui| {
            ui.label(t::heading("Reference maker", t::SECTION_TITLE));
            ui.add(
                egui::TextEdit::singleline(&mut self.makers.ref_source)
                    .desired_width(t::PROJECT_SOURCE_WIDTH)
                    .hint_text("Raw recording"),
            );
            if ui.button("Choose…").clicked() {
                picked = rfd::FileDialog::new().pick_file();
            }
            ui.add(
                egui::DragValue::new(&mut self.makers.seconds)
                    .range(3.0..=12.0)
                    .speed(0.1),
            );
            let busy = self.makers.job.is_some();
            find = t::primary_button(ui, "Find sentence runs", None, !busy)
                .on_disabled_hover_text("A tool is already running")
                .clicked();
        });
        self.job_line(ui, ToolKind::Candidates);
        crate::phase_l::bordered_columns(
            ui,
            t::PROJECT_CLIPS_FRACTION,
            ["reference_runs", "reference_expand"],
            |columns| {
                egui::ScrollArea::vertical()
                    .id_salt("ref_left")
                    .show(&mut columns[0], |ui| {
                        self.candidate_table(ui, &mut play_row);
                    });
                egui::ScrollArea::vertical()
                    .id_salt("ref_right")
                    .show(&mut columns[1], |ui| {
                        self.expanded_candidate(ui, &mut play_span);
                        self.reference_result(ui, &mut reveal, &mut use_ref);
                    });
            },
        );
        let ready = !self.makers.ref_name.trim().is_empty() && self.makers.job.is_none();
        self.makers.footer_status =
            self.makers.notice.clone().unwrap_or_else(|| {
                "Source recording is kept. Voice assignment is separate.".into()
            });
        self.makers.footer_primary = "Write reference".into();
        self.makers.footer_enabled = ready;
        if let Some(path) = picked {
            self.makers.ref_source = path.display().to_string();
            self.makers.candidates.clear();
            self.makers.written = None;
        }
        if find {
            let source = self.makers.ref_source.trim().to_owned();
            if source.is_empty() {
                self.makers.notice = Some("Choose a recording.".into());
            } else {
                self.launch_tool(
                    ToolKind::Candidates,
                    reference_candidates_invocation(&source, self.makers.seconds),
                );
            }
        }
        if let Some(index) = play_row {
            let source = self.makers.ref_source.clone();
            let span = self
                .makers
                .candidates
                .get(index)
                .map(|row| (row.start, row.end));
            if let Some((start, end)) = span {
                self.audition_project_media(Path::new(&source), start, Some(end));
            }
        }
        if play_span {
            self.play_reference_span();
        }
        if reveal && let Some(written) = &self.makers.written {
            crate::phase_j::reveal(Path::new(&written.wav));
        }
        if use_ref {
            self.use_written_reference();
        }
    }

    fn candidate_table(&mut self, ui: &mut Ui, play_row: &mut Option<usize>) {
        egui::Grid::new("reference_candidates")
            .striped(true)
            .num_columns(5)
            .show(ui, |ui| {
                for label in [
                    "Rank",
                    "Time range / duration",
                    "Sentence text",
                    "Score / 100",
                    "Play",
                ] {
                    ui.label(t::text(label, t::CAPTION, t::TEXT_MUTED));
                }
                ui.end_row();
                let rows = self.makers.candidates.clone();
                for (index, row) in rows.iter().enumerate() {
                    let selected = self.makers.expanded == Some(index);
                    if ui
                        .selectable_label(selected, format!("{:02}", row.rank))
                        .clicked()
                    {
                        self.makers.expanded = Some(index);
                        self.makers.ref_start = seconds_arg(row.start);
                        self.makers.ref_end = seconds_arg(row.end);
                        self.makers.transcript = row.text.clone();
                    }
                    ui.label(t::text(
                        format!(
                            "{} → {}\n{:.1} s",
                            clock(row.start),
                            clock(row.end),
                            (row.end - row.start).max(0.0)
                        ),
                        t::CAPTION,
                        t::TEXT_MUTED,
                    ));
                    ui.label(t::text(&row.text, t::BODY, t::TEXT_PRIMARY));
                    ui.vertical(|ui| {
                        ui.label(t::text(
                            format!("{:.0}", row.score * 100.0),
                            t::BODY,
                            t::TEXT_PRIMARY,
                        ));
                        meter(ui, row.score, false);
                    });
                    if ui.button("Play").clicked() {
                        *play_row = Some(index);
                    }
                    ui.end_row();
                }
            });
    }

    fn expanded_candidate(&mut self, ui: &mut Ui, play: &mut bool) {
        if self.makers.expanded.is_none() {
            return;
        }
        ui.separator();
        ui.label(t::text("Selected candidate", t::BODY, t::TEXT_PRIMARY));
        let path = PathBuf::from(&self.makers.ref_source);
        let start: f64 = self.makers.ref_start.parse().unwrap_or(0.0);
        let end: f64 = self.makers.ref_end.parse().unwrap_or(start);
        if path.is_file() {
            self.ensure_wave(&path, end.max(0.05), ui.ctx());
        }
        self.paint_wave(ui, end.max(start + 0.1), start, end, false);
        ui.horizontal(|ui| {
            ui.label(t::text("Start", t::CAPTION, t::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.makers.ref_start)
                    .desired_width(t::TIME_FIELD_WIDTH),
            );
            ui.label(t::text("End", t::CAPTION, t::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.makers.ref_end)
                    .desired_width(t::TIME_FIELD_WIDTH),
            );
            *play = ui.button("Play").clicked();
        });
        ui.label(t::text("Exact transcript", t::CAPTION, t::TEXT_MUTED));
        ui.text_edit_multiline(&mut self.makers.transcript);
        ui.horizontal(|ui| {
            ui.label(t::text("Reference name", t::CAPTION, t::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.makers.ref_name)
                    .desired_width(t::TIME_FIELD_WIDTH),
            );
            ui.label(t::text("Peak", t::CAPTION, t::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.makers.peak_target)
                    .desired_width(t::TIME_FIELD_WIDTH),
            );
        });
        self.job_line(ui, ToolKind::WriteReference);
    }

    fn reference_result(&mut self, ui: &mut Ui, reveal: &mut bool, use_ref: &mut bool) {
        let Some(written) = self.makers.written.clone() else {
            return;
        };
        ui.separator();
        ui.label(t::text("Reference written", t::BODY, t::TEXT_PRIMARY));
        ui.label(t::text(&written.wav, t::CAPTION, t::TEXT_MUTED));
        ui.label(
            egui::RichText::new(&written.text)
                .font(t::mono(t::CAPTION))
                .color(t::TEXT_PRIMARY),
        );
        ui.label(t::text(
            format!("peak {:.3} · {:.2} s", written.peak, written.duration),
            t::CAPTION,
            t::TEXT_MUTED,
        ));
        let name = self.speaker_name();
        ui.horizontal(|ui| {
            *reveal = t::secondary_button(ui, "Reveal files", None, true).clicked();
            *use_ref =
                t::primary_button(ui, &format!("Use for {name} in Voices"), None, true).clicked();
        });
    }

    fn audition_body(&mut self, ui: &mut Ui, report: &Report) {
        let mut run = false;
        let mut play: Option<String> = None;
        ui.label(t::heading("Audition", t::SECTION_TITLE));
        self.audition_header(ui, report, &mut run);
        self.job_line(ui, ToolKind::Audition);
        self.audition_columns(ui, &mut play);
        let name = speaker_name_in(report, &self.makers.audition_speaker);
        let ready = self.makers.audition_result.is_some() && self.makers.job.is_none();
        self.makers.footer_status = self.makers.notice.clone().unwrap_or_else(|| {
            "Winner is the highest mean. Underlined words differ from the sentence.".into()
        });
        self.makers.footer_primary = format!("Use this reference for {name}");
        self.makers.footer_enabled = ready;
        if run {
            self.begin_audition();
        }
        if let Some(path) = play {
            self.audition_project_media(Path::new(&path), 0.0, None);
        }
    }

    fn audition_header(&mut self, ui: &mut Ui, report: &Report, run: &mut bool) {
        let people: Vec<Person> = people_of(report)
            .into_iter()
            .filter(|person| {
                crate::project::engine_fields(&person.engine).contains(&"reference_audio")
            })
            .collect();
        let ids: Vec<&str> = people.iter().map(|person| person.id.as_str()).collect();
        let next =
            default_audition_speaker(&ids, &self.project.select_id, &self.makers.audition_speaker);
        if self.makers.audition_speaker != next {
            self.makers.audition_speaker = next;
        }
        ui.horizontal(|ui| {
            let label = people
                .iter()
                .find(|person| person.id == self.makers.audition_speaker)
                .map(|person| person.name.clone())
                .unwrap_or_else(|| "Speaker".into());
            egui::ComboBox::from_id_salt("audition_speaker")
                .selected_text(label)
                .show_ui(ui, |ui| {
                    for person in &people {
                        ui.selectable_value(
                            &mut self.makers.audition_speaker,
                            person.id.clone(),
                            &person.name,
                        );
                    }
                });
            let files = self.reference_files_in(report);
            for path in files {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let stored = path.display().to_string();
                let on = self.makers.chips.iter().any(|chip| chip == &stored);
                if ui.selectable_label(on, name).clicked() {
                    if on {
                        self.makers.chips.retain(|chip| chip != &stored);
                    } else if self.makers.chips.len() < 4 {
                        self.makers.chips.push(stored);
                    }
                }
            }
            let count = self.makers.chips.len();
            let ready = (2..=4).contains(&count) && self.makers.job.is_none();
            *run = t::primary_button(ui, "Run audition", None, ready)
                .on_disabled_hover_text("Choose 2–4 references")
                .clicked();
        });
        ui.label(t::text(
            "Underlined words differ from the sentence. The large number is mean × 100.",
            t::CAPTION,
            t::TEXT_MUTED,
        ));
    }

    fn audition_columns(&mut self, ui: &mut Ui, play: &mut Option<String>) {
        let Some(result) = self.makers.audition_result.clone() else {
            return;
        };
        let means: Vec<f64> = result.candidates.iter().map(|c| c.mean).collect();
        let winner = winner_index(&means);
        let count = result.candidates.len();
        if count == 0 {
            return;
        }
        ui.columns(count, |columns| {
            for (index, column) in columns.iter_mut().enumerate() {
                let Some(candidate) = result.candidates.get(index) else {
                    continue;
                };
                paint_audition_card(column, candidate, winner == Some(index), play);
            }
        });
    }

    fn m4_body(&mut self, ui: &mut Ui, report: &Report) {
        let mut run = false;
        let mut play: Option<String> = None;
        let mut pick: Option<usize> = None;
        let mut find = false;
        let episode = self
            .episode
            .as_ref()
            .map(|episode| episode.manifest.episode_id.clone())
            .unwrap_or_else(|| "episode".into());
        ui.label(t::heading(
            format!("Calibrate pronunciation  |  Outliers · {episode}"),
            t::SECTION_TITLE,
        ));
        crate::phase_l::bordered_columns(
            ui,
            t::PROJECT_CLIPS_FRACTION,
            ["calibrate_panel", "outliers_panel"],
            |columns| {
                egui::ScrollArea::vertical()
                    .id_salt("cal_scroll")
                    .show(&mut columns[0], |ui| {
                        self.calibrate_panel(ui, report, &mut run, &mut play);
                    });
                egui::ScrollArea::vertical()
                    .id_salt("out_scroll")
                    .show(&mut columns[1], |ui| {
                        self.outliers_panel(ui, &mut pick, &mut find);
                    });
            },
        );
        let best = self
            .makers
            .calibration
            .as_ref()
            .map(|result| result.best.clone())
            .unwrap_or_default();
        self.makers.footer_status = self
            .makers
            .notice
            .clone()
            .unwrap_or_else(|| "Write keeps the comments already in the file.".into());
        self.makers.footer_primary = if best.is_empty() {
            "Write to pronunciations.yaml".into()
        } else {
            format!("Write '{best}' to pronunciations.yaml")
        };
        self.makers.footer_enabled = !best.is_empty() && self.makers.job.is_none();
        if run {
            self.begin_calibrate(false);
        }
        if let Some(path) = play {
            self.audition_project_media(Path::new(&path), 0.0, None);
        }
        if let Some(row) = pick {
            self.makers.outlier_row = Some(row);
            self.select_outlier(row);
        }
        if find {
            self.makers.outlier_report = None;
            self.open_outliers();
        }
    }

    fn calibrate_panel(
        &mut self,
        ui: &mut Ui,
        report: &Report,
        run: &mut bool,
        play: &mut Option<String>,
    ) {
        labeled(ui, "Word", &mut self.makers.word);
        let readings = reading_options(report, &self.makers.word);
        if readings.is_empty() {
            labeled(ui, "Reading", &mut self.makers.reading);
        } else if self.makers.reading.is_empty()
            && let Some((key, _)) = readings.first()
        {
            self.makers.reading = key.clone();
        }
        for (key, spoken) in &readings {
            let label = format!("{key} · {spoken}");
            ui.radio_value(&mut self.makers.reading, key.clone(), label);
        }
        labeled(ui, "Candidate spellings", &mut self.makers.spellings);
        let people: Vec<Person> = people_of(report)
            .into_iter()
            .filter(|person| person.engine == "omnivoice")
            .collect();
        let label = people
            .iter()
            .find(|person| person.id == self.makers.cal_speaker)
            .map(|person| person.name.clone())
            .unwrap_or_else(|| "Speaker".into());
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("calibrate_speaker")
                .selected_text(label)
                .show_ui(ui, |ui| {
                    for person in &people {
                        ui.selectable_value(
                            &mut self.makers.cal_speaker,
                            person.id.clone(),
                            &person.name,
                        );
                    }
                });
            let ready = !self.makers.word.trim().is_empty()
                && !self.makers.reading.trim().is_empty()
                && !self.makers.cal_speaker.is_empty()
                && self.makers.job.is_none();
            *run = t::primary_button(ui, "Run calibration", None, ready)
                .on_disabled_hover_text("Word, reading, and an OmniVoice speaker are required")
                .clicked();
        });
        self.job_line(ui, ToolKind::Calibrate);
        let Some(result) = self.makers.calibration.clone() else {
            return;
        };
        let best = result.best.clone();
        egui::Grid::new("calibrate_grid")
            .striped(true)
            .show(ui, |ui| {
                for title in ["Spelling", "Score / 100", "Take 1", "Take 2", "Take 3"] {
                    ui.label(t::text(title, t::CAPTION, t::TEXT_MUTED));
                }
                ui.end_row();
                for row in &result.candidates {
                    let highlight = row.spelling == best;
                    let colour = if highlight {
                        t::accent()
                    } else {
                        t::TEXT_PRIMARY
                    };
                    ui.label(t::text(&row.spelling, t::BODY, colour));
                    ui.label(t::text(
                        format!("{:.1}", row.score * 100.0),
                        t::BODY,
                        colour,
                    ));
                    for slot in 0..3 {
                        if let Some(take) = row.takes.get(slot) {
                            let exists = Path::new(&take.wav).is_file();
                            let mark = if take.ok { "✓" } else { "✗" };
                            if ui
                                .add_enabled(exists, egui::Button::new(format!("Play {mark}")))
                                .on_disabled_hover_text("Audio file is missing")
                                .clicked()
                            {
                                *play = Some(take.wav.clone());
                            }
                        } else {
                            ui.label(t::text("—", t::CAPTION, t::TEXT_MUTED));
                        }
                    }
                    ui.end_row();
                }
            });
    }

    fn outliers_panel(&mut self, ui: &mut Ui, pick: &mut Option<usize>, find: &mut bool) {
        if self.makers.outlier_report.is_none() && ui.button("Find outliers").clicked() {
            *find = true;
        }
        self.job_line(ui, ToolKind::Outliers);
        let Some(report) = self.makers.outlier_report.clone() else {
            return;
        };
        egui::Grid::new("outliers_table")
            .striped(true)
            .show(ui, |ui| {
                for title in ["Turn / speaker", "Pred. s", "Actual s", "Ratio", "Reason"] {
                    ui.label(t::text(title, t::CAPTION, t::TEXT_MUTED));
                }
                ui.end_row();
                for (index, row) in report.outliers.iter().enumerate() {
                    let danger = classify_ratio(row.ratio) == RatioClass::Danger;
                    let selected = self.makers.outlier_row == Some(index);
                    if ui
                        .selectable_label(
                            selected,
                            format!("{:03}  {}", row.turn_index, row.speaker_id),
                        )
                        .clicked()
                    {
                        *pick = Some(index);
                    }
                    ui.label(t::text(
                        format!("{:.1}", row.predicted_s),
                        t::BODY,
                        t::TEXT_PRIMARY,
                    ));
                    ui.label(t::text(
                        format!("{:.1}", row.actual_s),
                        t::BODY,
                        t::TEXT_PRIMARY,
                    ));
                    ui.vertical(|ui| {
                        meter(ui, (row.ratio / 3.0).clamp(0.0, 1.0), danger);
                        let colour = if danger {
                            t::STATUS_FAILED
                        } else {
                            t::TEXT_PRIMARY
                        };
                        ui.label(t::text(format!("{:.2}×", row.ratio), t::CAPTION, colour));
                    });
                    let colour = if danger {
                        t::STATUS_FAILED
                    } else {
                        t::TEXT_MUTED
                    };
                    ui.label(t::text(&row.reason, t::CAPTION, colour));
                    ui.end_row();
                }
            });
        if let Some(index) = self.makers.outlier_row
            && let Some(row) = report.outliers.get(index)
        {
            ui.separator();
            ui.label(t::text(
                format!("Selected turn {}", row.turn_index),
                t::BODY,
                t::TEXT_PRIMARY,
            ));
            ui.label(t::text(&row.reason, t::BODY, t::TEXT_PRIMARY));
            ui.label(t::text(
                format!(
                    "{:.1} s actual vs {:.1} s predicted",
                    row.actual_s, row.predicted_s
                ),
                t::CAPTION,
                t::TEXT_MUTED,
            ));
        }
    }

    fn stress_window(&mut self, ctx: &egui::Context) {
        if !self.makers.stress {
            return;
        }
        let mut open = true;
        egui::Window::new("Stress check")
            .open(&mut open)
            .default_width(t::CLIP_PICKER_WIDTH)
            .resizable(true)
            .show(ctx, |ui| {
                self.job_line(ui, ToolKind::Stress);
                if let Some(report) = self.makers.stress_report.clone() {
                    for item in &report.heteronyms {
                        ui.label(t::text(&item.word, t::BODY, t::TEXT_PRIMARY));
                        ui.label(t::text(&item.expected, t::CAPTION, t::TEXT_MUTED));
                        ui.label(
                            egui::RichText::new(&item.heard)
                                .font(t::mono(t::BODY))
                                .color(if item.ok {
                                    t::STATUS_PASSED
                                } else {
                                    t::STATUS_FAILED
                                }),
                        );
                        ui.label(t::text(
                            if item.ok { "✓" } else { "✗" },
                            t::BODY,
                            if item.ok {
                                t::STATUS_PASSED
                            } else {
                                t::STATUS_FAILED
                            },
                        ));
                    }
                    if report.heteronyms.is_empty() {
                        ui.label(t::text(
                            "No heteronyms in this turn.",
                            t::CAPTION,
                            t::TEXT_MUTED,
                        ));
                    }
                }
            });
        self.makers.stress = open;
    }

    fn select_outlier(&mut self, row: usize) {
        let Some(turn_index) = self
            .makers
            .outlier_report
            .as_ref()
            .and_then(|report| report.outliers.get(row))
            .map(|outlier| outlier.turn_index)
        else {
            return;
        };
        let position = self.episode.as_ref().and_then(|episode| {
            episode
                .turns
                .iter()
                .position(|turn| turn.index == turn_index)
        });
        if let Some(position) = position {
            self.select(position);
        }
    }

    fn begin_audition(&mut self) {
        let speaker_id = self.makers.audition_speaker.clone();
        let people = self.people(|person| person.id == speaker_id);
        let Some(person) = people.into_iter().next() else {
            self.makers.notice = Some("Choose a speaker.".into());
            return;
        };
        let temperature = self.speaker_temperature(&speaker_id);
        let keep = self.ensure_keep(true);
        let call = audition_invocation(
            &self.makers.chips,
            &speaker_id,
            &person.engine,
            &keep.display().to_string(),
            temperature.as_deref(),
        );
        self.launch_tool(ToolKind::Audition, call);
    }

    fn begin_calibrate(&mut self, write: bool) {
        let keep = self.ensure_keep(false);
        let call = calibrate_invocation(
            &self.makers.word,
            &self.makers.reading,
            &self.makers.cal_speaker,
            &self.makers.spellings,
            &keep.display().to_string(),
            write,
        );
        self.launch_tool(ToolKind::Calibrate, call);
    }

    fn begin_reference_write(&mut self) {
        let start: f64 = self.makers.ref_start.parse().unwrap_or(0.0);
        let end: f64 = self.makers.ref_end.parse().unwrap_or(0.0);
        let peak: f64 = self.makers.peak_target.parse().unwrap_or(0.92);
        let call = reference_write_invocation(
            self.makers.ref_source.trim(),
            self.makers.ref_name.trim(),
            start,
            end,
            peak,
        );
        self.makers.written = None;
        self.launch_tool(ToolKind::WriteReference, call);
    }

    fn play_reference_span(&mut self) {
        let start: f64 = self.makers.ref_start.parse().unwrap_or(0.0);
        let end: f64 = self.makers.ref_end.parse().unwrap_or(start);
        let source = self.makers.ref_source.clone();
        self.audition_project_media(Path::new(&source), start, Some(end));
    }

    fn use_written_reference(&mut self) {
        let Some(written) = self.makers.written.clone() else {
            return;
        };
        let speaker = if self.makers.audition_speaker.is_empty() {
            self.project.select_id.clone()
        } else {
            self.makers.audition_speaker.clone()
        };
        let audio = self.stored_media(&written.wav);
        self.assign_voice(&speaker, &audio, Some(&written.text));
    }

    fn use_audition_reference(&mut self, index: usize) {
        let Some(candidate) = self
            .makers
            .audition_result
            .as_ref()
            .and_then(|result| result.candidates.get(index))
            .cloned()
        else {
            return;
        };
        let audio = self.stored_media(&candidate.reference);
        let sidecar = Path::new(&candidate.reference).with_extension("txt");
        let text = std::fs::read_to_string(&sidecar).ok();
        let speaker = self.makers.audition_speaker.clone();
        self.assign_voice(&speaker, &audio, text.as_deref());
        if text.is_none() {
            self.makers.notice = Some(
                "No transcript beside the reference. reference_audio changed; reference_text stayed."
                    .into(),
            );
        }
    }

    fn assign_voice(&mut self, speaker_id: &str, audio: &str, text: Option<&str>) {
        let Some(report) = self.project.draft.as_mut() else {
            self.makers.notice = Some("Open project data before assigning a reference.".into());
            return;
        };
        let Some(fields) = report
            .voices
            .speakers
            .iter_mut()
            .find(|speaker| speaker["id"].as_str() == Some(speaker_id))
            .and_then(|speaker| speaker["fields"].as_object_mut())
        else {
            self.makers.notice = Some("That speaker is not in the Voices draft.".into());
            return;
        };
        match text {
            Some(text) => assign_reference(fields, audio, text),
            None => {
                fields.insert(
                    "reference_audio".into(),
                    serde_json::Value::String(audio.into()),
                );
            }
        }
        self.project.tab = "voices".into();
        self.project.data_open = true;
    }

    fn ensure_keep(&mut self, audition: bool) -> PathBuf {
        let slot = if audition {
            &mut self.makers.keep_audition
        } else {
            &mut self.makers.keep_calibrate
        };
        if let Some(path) = slot.clone() {
            return path;
        }
        let path = std::env::temp_dir().join(format!(
            "scriptcast-m-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let _ = std::fs::create_dir_all(&path);
        *slot = Some(path.clone());
        path
    }

    fn ensure_wave(&mut self, path: &Path, duration: f64, ctx: &egui::Context) {
        if self.makers.wave_path.as_deref() == Some(path)
            && (self.makers.wave.is_some() || self.makers.wave_rx.is_some())
        {
            return;
        }
        self.makers.wave_path = Some(path.to_path_buf());
        self.makers.wave = None;
        self.makers.wave_origin = 0.0;
        let path = path.to_path_buf();
        let (tx, rx) = std::sync::mpsc::channel();
        self.makers.wave_rx = Some(rx);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = crate::waveform::clip_envelope(&path, 0.0, duration.max(0.05));
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }

    fn paint_wave(&mut self, ui: &mut Ui, duration: f64, start: f64, end: f64, snap_words: bool) {
        if self.makers.wave.is_none() && self.makers.wave_rx.is_some() {
            ui.label(t::text("Loading waveform…", t::CAPTION, t::TEXT_MUTED));
            return;
        }
        let envelope = match &self.makers.wave {
            Some(Ok(envelope)) if !envelope.peaks.is_empty() => crate::waveform::Envelope {
                peaks: envelope.peaks.clone(),
            },
            Some(Err(error)) => {
                ui.label(t::text(
                    format!("Cannot decode source audio: {error}"),
                    t::CAPTION,
                    t::STATUS_FAILED,
                ));
                return;
            }
            _ => {
                ui.label(t::text(
                    "Waveform appears when the source can be decoded.",
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
                return;
            }
        };
        let duration = duration.max(f64::from(crate::waveform::BUCKET_SECS));
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), t::PROJECT_WAVE_HEIGHT),
            egui::Sense::hover(),
        );
        ui.painter()
            .rect_filled(rect, t::RADIUS_CONTROL, t::BG_CONTROL);
        let full = crate::waveform::covers_window(&envelope.peaks, duration);
        let (frac0, frac1) = if full {
            (0.0_f32, 1.0_f32)
        } else {
            let covered = self.makers.wave_origin
                + envelope.peaks.len() as f64 * f64::from(crate::waveform::BUCKET_SECS);
            (
                (self.makers.wave_origin / duration).clamp(0.0, 1.0) as f32,
                (covered / duration).clamp(0.0, 1.0) as f32,
            )
        };
        let span = ((frac1 - frac0) * rect.width()).max(t::WAVE_BAR + t::WAVE_GAP);
        let bars =
            crate::waveform::bars(&envelope, (span / (t::WAVE_BAR + t::WAVE_GAP)) as usize);
        let left = rect.left() + frac0 * rect.width();
        for (index, peak) in bars.iter().enumerate() {
            let x = left + index as f32 * (t::WAVE_BAR + t::WAVE_GAP);
            let half = peak * rect.height() * t::PROJECT_WAVE_AMPLITUDE;
            ui.painter().line_segment(
                [
                    egui::pos2(x, rect.center().y - half),
                    egui::pos2(x, rect.center().y + half),
                ],
                egui::Stroke::new(t::WAVE_BAR, t::TEXT_MUTED),
            );
        }
        let x_at =
            |time: f64| rect.left() + ((time / duration).clamp(0.0, 1.0) as f32) * rect.width();
        let shade = egui::Rect::from_min_max(
            egui::pos2(x_at(start), rect.top()),
            egui::pos2(x_at(end), rect.bottom()),
        );
        ui.painter()
            .rect_filled(shade, t::RADIUS_CONTROL, t::BG_SELECTED);
        let words = self.makers.words.clone();
        for (key, edge, x) in [("start", true, x_at(start)), ("end", false, x_at(end))] {
            let handle = egui::Rect::from_center_size(
                egui::pos2(x, rect.center().y),
                egui::vec2(t::SPLITTER_HIT, rect.height()),
            );
            ui.painter()
                .rect_filled(handle, t::RADIUS_CONTROL, t::accent());
            let response = ui.interact(handle, ui.id().with(("m-wave", key)), egui::Sense::drag());
            if snap_words
                && response.dragged()
                && let Some(pos) = response.interact_pointer_pos()
                && let Some(words) = &words
            {
                let seconds =
                    f64::from(((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0)) * duration;
                if let Some(span) = self.makers.span {
                    let (lo, hi) = span.inclusive();
                    let next = if edge {
                        snap_start_index(&words.words, seconds, hi)
                    } else {
                        snap_end_index(&words.words, seconds, lo)
                    };
                    self.makers.span = Some(if edge {
                        WordSpan {
                            anchor: next,
                            focus: hi,
                        }
                    } else {
                        WordSpan {
                            anchor: lo,
                            focus: next,
                        }
                    });
                }
            } else if !snap_words
                && response.dragged()
                && let Some(pos) = response.interact_pointer_pos()
            {
                let seconds =
                    f64::from(((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0)) * duration;
                if edge {
                    self.makers.ref_start = seconds_arg(seconds.min(end));
                } else {
                    self.makers.ref_end = seconds_arg(seconds.max(start));
                }
            }
        }
    }

    fn people(&self, keep: impl Fn(&Person) -> bool) -> Vec<Person> {
        self.project
            .draft
            .as_ref()
            .map(people_of)
            .unwrap_or_default()
            .into_iter()
            .filter(|person| keep(person))
            .collect()
    }

    fn clip_id_list(&self) -> Vec<String> {
        self.project
            .draft
            .as_ref()
            .map(|report| {
                report
                    .clips
                    .clips
                    .iter()
                    .filter_map(|clip| clip["id"].as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn reference_files_in(&self, report: &Report) -> Vec<PathBuf> {
        let relative = report
            .toml
            .values
            .get("references")
            .and_then(|value| value.as_str())
            .unwrap_or("references");
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(report.root.join(relative)) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                }
            }
        }
        files.sort();
        if files.is_empty() {
            files.extend(self.makers.chips.iter().map(PathBuf::from));
        }
        files
    }

    fn speaker_name(&self) -> String {
        let id = if !self.makers.audition_speaker.is_empty() {
            self.makers.audition_speaker.as_str()
        } else {
            self.project.select_id.as_str()
        };
        self.people(|person| person.id == id)
            .into_iter()
            .next()
            .map(|person| person.name)
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "speaker".into())
    }

    fn speaker_temperature(&self, id: &str) -> Option<String> {
        self.project.draft.as_ref().and_then(|report| {
            report
                .voices
                .speakers
                .iter()
                .find(|speaker| speaker["id"].as_str() == Some(id))
                .and_then(|speaker| speaker["fields"].get("temperature"))
                .and_then(|value| {
                    value
                        .as_f64()
                        .map(|n| format!("{n:.2}"))
                        .or_else(|| value.as_str().map(str::to_owned))
                })
        })
    }

    fn stored_media(&self, path: &str) -> String {
        let Some(root) = self.project.root.clone().or_else(|| {
            self.project
                .draft
                .as_ref()
                .map(|report| report.root.clone())
        }) else {
            return path.to_owned();
        };
        crate::phase_l::stored_path(&root, PathBuf::from(path))
    }

    fn selected_turn_label(&self) -> String {
        self.selected
            .and_then(|index| self.episode.as_ref().and_then(|e| e.turns.get(index)))
            .map(|turn| format!("turn {:03}", turn.index))
            .unwrap_or_else(|| "the selected turn".into())
    }
}

enum Outcome {
    Words(WordsJson),
    Registered(ClipRegistered),
    Candidates(CandidatesJson),
    Written(ReferenceWritten),
    Audition(AuditionJson),
    Calibration(CalibrateJson),
    Outliers(OutliersJson),
    Stress(StressJson),
}

fn apply_tool_json(kind: ToolKind, bytes: &[u8]) -> Result<Outcome, String> {
    Ok(match kind {
        ToolKind::Words => Outcome::Words(parse_tool(bytes)?),
        ToolKind::Register => Outcome::Registered(parse_tool(bytes)?),
        ToolKind::Candidates => Outcome::Candidates(parse_tool(bytes)?),
        ToolKind::WriteReference => Outcome::Written(parse_tool(bytes)?),
        ToolKind::Audition => Outcome::Audition(parse_tool(bytes)?),
        ToolKind::Calibrate => Outcome::Calibration(parse_tool(bytes)?),
        ToolKind::Outliers => Outcome::Outliers(parse_tool(bytes)?),
        ToolKind::Stress => Outcome::Stress(parse_tool(bytes)?),
    })
}

fn read_fixture<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    parse_tool(&std::fs::read(path).map_err(|e| e.to_string())?)
}

fn speaker_name_in(report: &Report, id: &str) -> String {
    people_of(report)
        .into_iter()
        .find(|person| person.id == id)
        .map(|person| person.name)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "speaker".into())
}

fn people_of(report: &Report) -> Vec<Person> {
    report
        .voices
        .speakers
        .iter()
        .filter_map(|speaker| {
            let id = speaker["id"].as_str()?.to_owned();
            let fields = speaker.get("fields");
            let name = fields
                .and_then(|f| f.get("name"))
                .and_then(|v| v.as_str())
                .filter(|name| !name.is_empty())
                .or(speaker["name"].as_str())
                .unwrap_or(&id)
                .to_owned();
            let engine = fields
                .and_then(|f| f.get("engine"))
                .and_then(|v| v.as_str())
                .or(speaker["engine"].as_str())
                .unwrap_or("kokoro")
                .to_owned();
            Some(Person { id, name, engine })
        })
        .collect()
}

fn step_row(ui: &mut Ui, current: u8) {
    ui.horizontal(|ui| {
        for (index, label) in [(1, "1 Source"), (2, "2 Pick words"), (3, "3 Details")] {
            let colour = if index == current {
                t::accent()
            } else {
                t::TEXT_MUTED
            };
            ui.label(t::text(label, t::BODY, colour));
            if index < 3 {
                ui.label(t::text("·", t::BODY, t::TEXT_MUTED));
            }
        }
    });
}

fn flowing_words(ui: &mut Ui, words: &[TimedWord], span: Option<WordSpan>) -> Option<WordSpan> {
    let full = words
        .iter()
        .map(|word| word.word.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let mut offsets = Vec::with_capacity(words.len());
    let mut cursor = 0usize;
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            cursor += 1;
        }
        offsets.push(cursor);
        cursor += word.word.len();
    }
    let (lo, hi) = span
        .map(|span| span.inclusive())
        .filter(|(lo, hi)| *lo < words.len() && *hi < words.len())
        .unwrap_or((usize::MAX, 0));
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = ui.available_width();
    let plain = egui::TextFormat {
        font_id: t::font(t::BODY),
        color: t::TEXT_PRIMARY,
        ..Default::default()
    };
    let marked = egui::TextFormat {
        font_id: t::font(t::BODY),
        color: t::TEXT_PRIMARY,
        background: t::BG_SELECTED,
        ..Default::default()
    };
    if lo <= hi {
        let mid_start = offsets[lo];
        let mid_end = offsets[hi] + words[hi].word.len();
        if mid_start > 0 {
            job.append(&full[..mid_start], 0.0, plain.clone());
        }
        job.append(&full[mid_start..mid_end], 0.0, marked);
        if mid_end < full.len() {
            job.append(&full[mid_end..], 0.0, plain);
        }
    } else {
        job.append(&full, 0.0, plain);
    }
    let galley = ui.ctx().fonts_mut(|fonts| fonts.layout_job(job.clone()));
    let response = ui.add(egui::Label::new(job).sense(egui::Sense::click()));
    let pos = response.interact_pointer_pos()?;
    if !response.clicked() {
        return None;
    }
    let local = pos - response.rect.min;
    let byte = usize::from(galley.cursor_from_pos(local).index);
    let index = offsets
        .iter()
        .rposition(|start| *start <= byte)
        .unwrap_or(0);
    let shift = ui.input(|input| input.modifiers.shift);
    Some(match span {
        Some(span) => span.click(index, shift),
        None => WordSpan {
            anchor: index,
            focus: index,
        },
    })
}

fn reading_options(report: &Report, word: &str) -> Vec<(String, String)> {
    let wanted = word.trim();
    report
        .pronunciations
        .entries
        .iter()
        .find(|entry| entry["word"].as_str() == Some(wanted))
        .and_then(|entry| entry["readings"].as_object())
        .map(|readings| {
            readings
                .iter()
                .map(|(key, value)| (key.clone(), value.as_str().unwrap_or("").to_owned()))
                .collect()
        })
        .unwrap_or_default()
}

fn labeled(ui: &mut Ui, label: &str, value: &mut String) {
    ui.label(t::text(label, t::CAPTION, t::TEXT_MUTED));
    ui.add(egui::TextEdit::singleline(value).desired_width(ui.available_width()));
}

fn meter(ui: &mut Ui, fraction: f64, danger: bool) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(t::METER_WIDTH, t::SPACE_4), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, t::RADIUS_SEEK_TRACK, t::BG_CONTROL);
    let mut fill = rect;
    fill.max.x = rect.left() + rect.width() * fraction.clamp(0.0, 1.0) as f32;
    ui.painter().rect_filled(
        fill,
        t::RADIUS_SEEK_TRACK,
        if danger {
            t::STATUS_FAILED
        } else {
            t::accent()
        },
    );
}

fn paint_audition_card(
    ui: &mut Ui,
    candidate: &AuditionCandidate,
    win: bool,
    play: &mut Option<String>,
) -> (f32, f32, f32) {
    let height = ui.available_height();
    let mut inner = 0.0;
    let mut heard_wrap = 0.0;
    let mut heard_galley = 0.0;
    egui::Frame::new()
        .fill(if win { t::BG_SELECTED } else { t::BG_SURFACE })
        .stroke(egui::Stroke::new(
            t::BORDER,
            if win { t::accent() } else { t::BORDER_DEFAULT },
        ))
        .corner_radius(t::RADIUS_CARD)
        .inner_margin(t::SPACE_12)
        .show(ui, |ui| {
            if height.is_finite() {
                ui.set_min_height(height);
            }
            ui.set_width(ui.available_width());
            inner = ui.available_width();
            let file = Path::new(&candidate.reference)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| candidate.reference.clone());
            ui.horizontal(|ui| {
                ui.label(t::text(&file, t::BODY, t::TEXT_PRIMARY));
                if win {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(t::text("Winner", t::CAPTION, t::accent()));
                    });
                }
            });
            let colour = match mean_band(candidate.mean) {
                MeanBand::High => t::STATUS_PASSED,
                MeanBand::Mid => t::TEXT_PRIMARY,
                MeanBand::Low => t::STATUS_WARNING,
            };
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(mean_percent(candidate.mean))
                        .font(t::bold(t::DIALOG_TITLE))
                        .color(colour),
                );
                ui.label(t::text(
                    format!("{}/{} takes", candidate.clean, candidate.of_takes),
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
            });
            for take in &candidate.takes {
                ui.label(t::text(&take.text, t::BODY, t::TEXT_PRIMARY));
                ui.label(t::text("WHISPER HEARD", t::CAPTION, t::TEXT_MUTED));
                (heard_wrap, heard_galley) = paint_heard(ui, &take.text, &take.heard);
                ui.horizontal(|ui| {
                    ui.label(t::text(
                        format!("score {:.0} · run {}", take.score * 100.0, take.run),
                        t::CAPTION,
                        t::TEXT_MUTED,
                    ));
                    let exists = Path::new(&take.wav).is_file();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(exists, egui::Button::new("Play"))
                            .on_disabled_hover_text("Audio file is missing")
                            .clicked()
                        {
                            *play = Some(take.wav.clone());
                        }
                    });
                });
            }
        });
    (inner, heard_wrap, heard_galley)
}

fn paint_heard(ui: &mut Ui, sentence: &str, heard: &str) -> (f32, f32) {
    let width = ui.available_width();
    let job = heard_job(sentence, heard, width);
    let galley = ui.ctx().fonts_mut(|fonts| fonts.layout_job(job.clone()));
    let galley_width = galley.rect.width();
    ui.add_sized(
        egui::vec2(width, galley.rect.height().max(t::BODY)),
        egui::Label::new(job),
    );
    (galley.job.wrap.max_width, galley_width)
}

fn heard_job(sentence: &str, heard: &str, width: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = width;
    let diff = heard_diff(sentence, heard);
    for (index, word) in diff.iter().enumerate() {
        if index > 0 {
            job.append(
                " ",
                0.0,
                egui::TextFormat {
                    font_id: t::font(t::BODY),
                    color: t::TEXT_PRIMARY,
                    ..Default::default()
                },
            );
        }
        job.append(
            &word.text,
            0.0,
            egui::TextFormat {
                font_id: t::font(t::BODY),
                color: if word.differs {
                    t::STATUS_FAILED
                } else {
                    t::TEXT_PRIMARY
                },
                underline: if word.differs {
                    egui::Stroke::new(t::BORDER, t::STATUS_FAILED)
                } else {
                    egui::Stroke::NONE
                },
                ..Default::default()
            },
        );
    }
    job
}

fn tail_log(path: &Path) -> String {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = std::fs::File::open(path) else {
        return String::new();
    };
    let len = file.metadata().map(|meta| meta.len()).unwrap_or(0);
    let start = len.saturating_sub(8192);
    if file.seek(SeekFrom::Start(start)).is_err() {
        return String::new();
    }
    let mut buf = String::new();
    if file.read_to_string(&mut buf).is_err() {
        return String::new();
    }
    if start > 0
        && let Some(index) = buf.find('\n')
    {
        buf.drain(..=index);
    }
    buf
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    #[test]
    fn heard_text_galley_matches_the_card_inner_width() {
        use crate::phase_m::{AuditionCandidate, Take};
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx);
        let candidate = AuditionCandidate {
            reference: "/tmp/aisha_news.wav".into(),
            temperature: 0.6,
            mean: 0.936,
            min: 0.9,
            clean: 3,
            of_takes: 3,
            takes: vec![Take {
                wav: "/tmp/keep/news_0.wav".into(),
                text: "We need to listen before we decide.".into(),
                heard: "We need to learn before we decide and the heard line keeps going \
                    so it wraps across the full inner width of the audition card."
                    .into(),
                score: 1.0,
                run: 0,
            }],
        };
        let mut inner = 0.0_f32;
        let mut wrap = 0.0_f32;
        let mut galley = 0.0_f32;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(960.0, 640.0),
                )),
                ..Default::default()
            },
            |ui| {
                ui.set_min_width(960.0);
                ui.columns(3, |columns| {
                    let mut play = None;
                    (inner, wrap, galley) =
                        super::paint_audition_card(&mut columns[0], &candidate, true, &mut play);
                });
            },
        );
        output.textures_delta.clear();
        assert!(
            (wrap - inner).abs() <= 1.0,
            "heard galley wrap {wrap} card inner {inner}"
        );
        assert!(
            galley + 16.0 >= inner,
            "heard galley {galley} card inner {inner}"
        );
    }
}
