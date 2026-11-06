# Tech Plan: Laya Router

Source plan: [plan.md](plan.md). Loops: [loop.md](loop.md).

## Architectural Approach

The Stage loop keeps its existing shape (stills → verify → dispatch) and gains four local services, each a CLI with a JSON file contract, in the same style as the audio tools. Every model-backed decision has a deterministic twin that decides first; the model's answer is logged beside it until a measured gate promotes it. Local models run on Apple Silicon through MLX in their own venvs, so a missing model degrades a CLI to its rules path and never blocks a render.

## Key Architectural Decisions

| # | Decision | Reason |
|---|---|---|
| D1 | Rules decide; laya shadows until promoted per question | Measured zero-shot quality on this machine is low (lane 3/4 at confidence ≤ 0.58; follow-up `noul` ≤ 0.20 on an obvious follow-up). Labels to fine-tune do not exist yet. |
| D2 | `session_action` is rules-only and never goes to laya | The registry knows whether a live builder session exists. A classifier adds error to a lookup. |
| D3 | Laya never picks a model name; it picks lane and tier, and `tiers.toml` maps them to agent + model | Quota pools and model names change weekly. A table edit absorbs that; a retrain does not. |
| D4 | Cue cards, never briefs, go to laya; ≤ 400 tokens | English checkpoint context is 512 tokens. |
| D5 | Two option orders per choice, averaged; disagreement → abstain | Notebook source (AnyJev) reports ~33% decision flips under option reversal. Cost is one extra ~40 ms pass. |
| D6 | Extend `stage_dispatch.py`; no new Rust wrapper | It already starts/resumes the four worker CLIs, captures session ids and writes the dispatch log. |
| D7 | Critic runs only on Tier 1 passes and adds a Content category | Tier 1 is cheap and deterministic. The four arXiv categories have no slot for on-screen text that departs from the storyboard, the most common author note. |
| D8 | The critic's verdict gates nothing until calibrated on YT02 frames | The 100% figure is one frontier VLM on HTML5 canvas apps; local 7B models on Stage frames are unmeasured. |
| D9 | nlm context is a cited passage with a size cap, cached by hash | Keeps briefs short and repeatable; notebooks are context, `.tex` and `.aux` stay the authority. |
| D10 | SVG offload is an experiment with a written decision rule as its output | The token-saving claim is unmeasured, and people and equations are already covered by house components. |
| D11 | Critic defects pass a cheap grounding check (declared element id + region overlap ≥ 30%, or OCR-present quote for Content); failures are dropped, no retry | Pattern from the Airlock's `groundedness.py`, reduced to box comparisons so it adds no time. Stops a builder chasing a defect the VLM invented. |
| D12 | Book and chapter notebook passages enter briefs as-is; public-research notebook passages are fenced as reference material | Internal tool; the book is trusted content. Fencing web-sourced text costs nothing. |

## Architecture Validation Notes

- laya-mlx 0.6.0 loads and answers on this M1 Max (27–51 ms per two-question call). 0.1.0 in the system Python is broken against `tokenizers 0.19.1`; the `router` extra pins 0.6.0 in `.venv-router`.
- The checkpoint warns that some temperature buckets are clamped and uncalibrated, so ECE is measured per question before any confidence threshold is trusted.
- `nlm` is installed and authenticated (`nlm notebook list` and `nlm notebook query` both worked 2026-11-06). A query takes tens of seconds, so context is fetched once per dispatch and cached.
- MLX work runs with the Bash sandbox disabled (vault note: the sandbox throttles Metal about 1000×). Worker loops never run MLX jobs inside the worker's own sandbox; the operator or the CLI runs them on the host.

## Failure Modes

| Failure | Detection | Behaviour |
|---|---|---|
| laya venv missing or model load fails | import/load exception | Log `laya: unavailable`; rules decide; exit 0 |
| Cue card over 400 tokens | `CardTooLong` | Truncate the note field to fit and set `card_truncated: true`; if still over, skip laya |
| Laya order disagreement or low confidence | D5 / `min_confidence` | Abstain; rules decide; the record is flagged for labelling |
| Rules have no matching row | no rule fired | Exit 3 with the card printed; the operator decides with `--override`, which becomes a label |
| `nlm` auth expired | non-zero exit, auth text | Exit 2 with the `nlm login` hint; dispatch proceeds without context only with `--no-context` |
| Session id stale (CLI cannot resume) | resume exit code / stderr signature | Registry marks the session `dead`; dispatch starts fresh seeded with the last summary |
| Worker CLI reports no tokens | missing field | Registry uses the brief-character sum as the load proxy and marks it `proxy` |
| VLM out of memory or slow | exception / > 60 s per frame | Critic writes `status: skipped` for the frame; Tier 1's result stands |
| Critic false fails flood repair | false-fail rate tracked per run | Above 0.1 on approved fixtures, `stage-repair` ignores critic defects and reports it |
| Critic names an undeclared element or a region off its box | `grounding.check` | Drop the defect and count it `ungrounded` |
| Critic output fails the JSON schema twice | schema validation | `status: contract_error`; repair treats it as no verdict |
| Generated SVG carries text | `<text>` nodes present | Run `check_antithesis.py` on it; reject numbers not traceable to `.aux` |

## Data Model

### Existing (consumed)

- `stage_plan.json` — segments, scenes, cues, frame ranges (`stage_stills.py`).
- `verify.json` — Tier 1 results per still (`stage_verify.py`).
- `review/notes.json` — author notes: scene, element id, stage ms, nearest cue, text (L3).
- Vault `Orchestration/dispatch-log.jsonl` — `ts, project, task, kind, agent, model, flags, brief_chars, wall_s, tokens, wrote_files, acceptance, iterations, verdict, failure_mode, note`.

### New (produced)

**Cue card**
```json
{"scene": "APAGroupDefinitions", "turn": 21, "cue": "t21Conduct", "element_id": "allocation-tray",
 "kind": "fix", "defect_category": "layout", "note": "tray sits 40 px too high, overlaps the card",
 "owned_files": ["stage/min3_7/a/APAGroupDefinitions.tsx"], "has_live_session": true}
```

**Router record** (`router_log.jsonl`)
```json
{"ts": "…", "card_sha": "…", "card_tokens": 112,
 "rules": {"lane": "tsx_scene", "tier": "standard", "session_action": "resume_builder", "context_source": "none", "rule_ids": ["L3", "S1"]},
 "laya": {"model": "aac6fef/laya-mlx", "version": "0.6.0", "ms": 81.2,
          "lane": {"fwd": ["tsx_scene", 0.41], "rev": ["tsx_scene", 0.37], "avg": ["tsx_scene", 0.39], "abstain": false},
          "tier": {"score": 1.2}, "context_source": {"avg": ["none", 0.62]}},
 "decided_by": {"lane": "rules", "tier": "rules", "context_source": "rules"},
 "override": null, "dispatch_ref": "<dispatch-log ts or session id>"}
```

**Session registry** (`sessions.json`)
```json
{"yt02/APAGroupDefinitions/builder": {"agent": "codex", "model": "gpt-5.6-sol", "session_id": "…",
  "turns": 4, "tokens": 14200, "tokens_kind": "reported|proxy", "context_limit": 400000,
  "status": "live|compacted|closed|dead", "summary_path": null, "last_active": "…"}}
```

**Critic verdict** (`critic.json`, per still)
```json
{"still": "APAGroupDefinitions__t21Conduct.png", "model": "qwen2.5-vl-7b-4bit", "ms": 9400,
 "status": "ok|skipped|contract_error", "verdict": "fail",
 "defects": [{"category": "layout", "element_id": "allocation-tray", "region": [0.43, 0.62, 0.50, 0.21],
 "evidence": "tray top edge overlaps the definition card by ~40 px", "severity": "major", "grounded": true}],
 "ungrounded_dropped": 0}
```

## Component Architecture

- **`router/cue_card.py`** — builds the card from a note id or flagged cue, the stage plan and the registry; counts tokens with the laya tokenizer when present, else a 4-chars-per-token estimate marked `estimated`.
- **`tools/stage_route.py`** — loads `rules.toml`, `questions.json`, `tiers.toml`; rules first; laya in shadow with two orders; writes the record; prints the decision as JSON for `stage_dispatch`.
- **`tools/stage_context.py`** — `nlm notebook query` subprocess (argument list, no shell), passage extraction from the `references` array, cache in `.router/context_cache/<sha>.json`.
- **`tools/stage_dispatch.py`** — registry read/write with a file lock; compaction message templates; `--context`; `close` subcommand.
- **`tools/stage_critic.py`** — mlx-vlm load once per run; one prompt per still built from `critic/prompt.md`, the cue's storyboard text and declared element ids, and the approved reference still when one exists; JSON-schema validation of output with one retry.
- **`tools/stage_repair.py`** — joins `verify.json`, `critic.json`, `review/notes.json`; groups by registry builder; one `resume` per builder; round counter in `sessions.json`.

## Component Boundaries

- Laya and the VLM never write files a worker owns. They emit decisions and verdicts only.
- `stage_route` never calls a worker CLI; `stage_dispatch` is the only process that launches workers.
- `stage_context` is the only process that calls `nlm`.
- MLX-backed CLIs run on the host shell, outside any worker sandbox.

## Integration Sequence

1. R0 environments and labels.
2. R1 router in shadow → operator uses it for every dispatch for at least one Stage pass (labels accumulate).
3. R2 registry + context; measured against current practice.
4. R3 critic in parallel with R1–R2 (disjoint files); calibrated before R4 uses it.
5. R4 repair, consuming the registry (R2) and critic (R3).
6. R5 SVG experiment with R3 as judge.
7. R6 fine-tune and per-question promotion.
8. v0.4 experiment X runs on R1–R4 tooling.

## R7: AI sidebar

The sidebar is the operator's way in to the router. It lives in the egui app (`ui/src/ai_sidebar.rs`) and knows the router only through one trait:

```rust
pub trait RouterClient {
    fn submit(&mut self, request: &RouterRequest) -> Delivery; // Queued | Failed(String)
    fn describe(&self) -> String;
}
```

| Decision | Choice | Why |
|---|---|---|
| Transport before R1 | Append-only `queue.jsonl` (`JsonlQueueClient`) | No router process yet; a file is inspectable and R1 can drain it |
| Queue location | `$SCRIPTCAST_ROUTER_QUEUE`, else `~/Library/Application Support/ScriptCast/router/queue.jsonl` | Outside episode folders, which are render outputs |
| Speech to text | `SpeechToText` trait, `SttStub` in R7 | mlx-whisper needs mic capture and a large model download; deferred to R7b |
| Layout | Fixed 320 px left strip; 36 px rail when collapsed | Leaves the existing screens' layout code untouched (they render into the remaining rect) |
| Tests | `MemoryClient` in the harness; sidebar hidden unless `--ai-sidebar` | Existing snapshots stay byte-identical |

Queue record (one per line):

```json
{"id":1,"text":"Tighten the pause after turn 2.","source":"typed","episode":"/path/to/ep","submitted_at":"2026-11-06T12:00:00Z"}
```

Failure modes: an unwritable queue path returns `Delivery::Failed`, shown as a `failed` status and a router reply with the IO error; nothing is retried. Requests with AI off or empty text are dropped before the client. Replies are local placeholders until R1 writes answers back (format to be set in R1).

Design: [Figma frame](https://www.figma.com/design/FyNyDUIlkH6Kjqvc3lcR2L?node-id=1-2).


### Decisions

| # | Decision | Reason |
|---|---|---|
| D13 | The Rust parser re-derives the still list from `stage_plan.json` exactly as `stage_stills.py:resolve_scene_stills` does (shot-prefixed cues, bare-cue fallback, cue + 700 ms, `ceil((ms − range.start_ms) × fps / 1000)` capped at segment end, `<scene>_<cue>.png` + `<scene>_end.png`) | `stills.json` only exists after a render pass and carries absolute paths from that machine; the plan is the durable contract both producers (`stage.py plan`, `stage-stills`) already agree on. One derivation, no second source of truth. |
| D14 | Turn sync keys on `turn_id` (shot → turn), never on row index | The browser can filter/reorder visible turns; `episode.turns` position and manifest `turn_index` are display facts, `turn_id` is the identity the plan, manifest, and `stress.json` all share. |
| D15 | Scroll→turn fires only on a user scroll-offset change, and a scroll-driven selection suppresses the answering programmatic scroll | Otherwise turn→scroll and scroll→turn chase each other every frame. The view remembers the last synced turn; an echo of the same turn is a no-op. |
| D16 | Ctrl+R is Control+R; Cmd+R keeps meaning Reload | macOS reserves Cmd chords for the app shell (`shortcuts()` already binds Cmd+R, Cmd+O, Cmd+F…). Control is free, and egui distinguishes the two modifiers exactly. |
| D17 | Sending is an explicit button press; toggling compact mode off never sends | A mode toggle is navigation, not consent to dispatch work. One Send = one `submit` = one JSONL line; an empty selection disables Send. |
| D18 | `RouterClient` is the R7-shared trait, `submit(&mut self, request: &RouterRequest) -> Delivery` + `describe()`; a stage selection travels as a `RouterRequest` with `source: Selection` whose `text` is the JSON of `SelectionContext` | R7 already defines this module (`Source::{Typed, Voice}`, `JsonlQueueClient`, `MemoryClient`); R8 adds only the `Selection` variant and `SelectionContext` so the branches merge. `JsonlQueueClient` (append one JSON line, create parent dirs) queues everything — no queue daemon, no watcher in this phase. |
| D19 | Missing stills and missing plans degrade to placeholders/notices | Still rendering is a separate, slower pass than plan compilation; the browser must stay useful when only one of them has run. Existing episodes without a plan render byte-identically (no column is carved out). |

### Data model (new, produced by the UI)

**SelectionContext** (`ui/src/router_client.rs`): one queue line per Send is a `RouterRequest`; its `text` is the JSON below (`source: "selection"`, `episode` is the episode directory, `submitted_at` from `router_client::now_string()`):

```json
{"episode": "fixture-ep", "turn_ids": ["turn-1", "turn-2"], "cue_ids": ["intro", "detail"], "frame_paths": ["/…/SceneA_intro.png", "/…/SceneA_detail.png"]}
```

`turn_ids` are deduplicated in frame order (clicking two cues of one turn names the turn once); `cue_ids` are the bare cue names from the canonical `<shot_id>.<cue>` keys (`end` for the segment-end frame); `frame_paths` are the resolved still paths in frame order, whether or not the file exists yet. `RouterRequest::selection(id, context, episode_dir, submitted_at)` builds the request; `request.selection_context()` parses it back.

### Components

- **`ui/src/stage_view.rs`** — `StagePlan::load(plan_path, stills_dir)` (serde mirror of `build_plan` output; unknown fields ignored, `words` skipped), ordered `StageFrame { scene, shot_id, turn_id, turn_index, cue_id, cue_key, ms, frame_index, file_name, path }` derivation, `StageView` state (`compact`, `selected_frames: BTreeSet<usize>`, pending scroll target, last synced turn, texture cache), and `show(ui, view, selected_turn_id, client) -> StageAction`. Pure helpers (`frames_for_turn`, `turn_for_frame`, `selection_context`, `frame_at_offset`) carry the sync logic and are unit-tested without egui. Textures decode through the existing `image` dependency and cache per path, as in `picture.rs`.
- **`ui/src/router_client.rs`** — shared with R7: `Source::{Typed, Voice, Selection}`, `RouterRequest { id, text, source, episode, submitted_at }`, `SelectionContext`, `RouterClient` (`submit(&mut self, &RouterRequest) -> Delivery`, `describe()`), `JsonlQueueClient { path }` appending `serde_json` + `\n` under `OpenOptions` create/append, and `MemoryClient` for tests. R8 adds `RouterRequest::selection(..)` / `selection_context()` and a `JsonlQueue` type alias.
- **`ui/src/app_main.rs` / `capture.rs`** — `Browser` gains `stage: Option<StageView>`, `stage_client: Option<JsonlQueueClient>`, and a load notice. `accept_episode` probes `<episode>/stage_plan.json` (or `--stage-plan`), stills default to the plan directory (or `--stage-stills`), queue defaults to `<episode>/router-queue.jsonl` (or `--stage-queue`). The review row carves a stage column from its right edge (340 pt normal, 180 pt compact) only when a plan loaded. `StageAction::SelectTurn` resolves `turn_id` → `episode.turns` position and calls the existing `select()`; `Submitted` is surfaced as a transient notice. Shortcuts are untouched: the stage view consumes Ctrl+R itself, after `shortcuts()` has run, and ignores it while a text edit has focus.

### Failure modes (R8 additions)

| Failure | Detection | Behaviour |
|---|---|---|
| No `stage_plan.json` in the episode dir | probe `is_file` false | No stage column; browser unchanged |
| Plan JSON malformed / no segments / no cues | serde / derivation error | Notice in the stage column header area; no column content, no crash; episode still loads |
| Still PNG missing or undecodable | `is_file` / decode error | Labelled placeholder tile with the file name; frame stays selectable and submittable |
| Queue path unwritable | `submit` returns `Delivery::Failed` | Send keeps the selection, shows the error as a notice; no partial line (single `writeln!` of the serialized request) |
| Ctrl+R while editing text | text-edit focus check | Ignored, matching every other browser shortcut |
| Selected turn has no cue frames | `frames_for_turn` empty | No scroll target; selection in the table is unaffected |

### Tests

- Unit (beside the code): plan derivation against the fixture plan (cue order, +700 ms frame math, end frame, shot→turn assignment), alias cues not double-counted, `selection_context` dedup/order, `JsonlQueueClient` appends one parseable `RouterRequest` line per submit and creates parent dirs.
- Snapshot: `stage_scroll` in `ui/tests/snapshots.rs` (browser at 1440×900 over `ui/tests/fixtures/stage/`, turn 1 selected); regenerate with `UPDATE_SNAPSHOTS=1 cargo test --test snapshots`.
- Interaction: `ui/tests/stage_view.rs` drives the real widget with `egui_kittest` — Ctrl+R shrinks the view (`compact` + smaller thumbnail height), clicking two frames across two turns selects both, Send produces exactly one JSONL line matching the expected `SelectionContext`, and a second Send appends a second line rather than rewriting.
