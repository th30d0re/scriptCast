# Stage production paradigm: how to orchestrate video work

Read this before planning or dispatching any Stage (code-driven video) work. It records what
YT02 "What Racism Is" (Episode 1) taught us between 2026-11-01 and 2026-11-02, and how the
pipeline is meant to run.

## 1. The precedent: audio solved this already

scriptCast reached long-form audio by iterating in text and audio, and by **verifying generated
output against ground truth with independent offline models**:
- `scriptcast-verify` runs on-device Whisper and compares the transcript to the script.
- `scriptcast-stress` uses a phoneme recognizer to catch what Whisper can't hear.
- `scriptcast-repair` re-rolls only the turns that failed and re-checks only those.

Video follows the same pattern.
- The ground truth is the **scene spec**.
- The read-back is OCR plus pixel measurement of rendered stills, with a vision model (VLM) on the
  flagged ones.
- The repair goes to the session that built the scene.

## 2. Why the old loop failed (measured)

Building 7:05–12:32 took 15 cold-start worker runs and still needed review. Codex used ~2.4 M
tokens over 18 runs for the first 12 minutes, and its fix passes cost more than its builds. The
causes:

1. **One pass did both design and code.** Workers invented the composition and typed its pixel
   geometry in the same 200–550-line component. Almost every defect was geometric: overlaps,
   oversized cards, text on the border, empty boxes.
2. **The checks measured text, not pictures.**
   - One model met a "box is full" check by padding boxes with filler labels (Argon).
   - Others stripped the labels and left boxes empty (Composer, Grok, Flash).
   - Neither was visible to the checks. Only rendered frames showed it.
3. **Every fix started cold.** Each run re-read 100–270k tokens to change a few dozen lines.
4. **The rules oscillated.** Each new rule was enforced at every frame, and each one produced a
   new set of failures.

## 3. Pass structure (one sequence ≈ 5 minutes of picture)

| Pass | Who | Output | Never |
|---|---|---|---|
| **P0 Storyboard** | Frontier model | Scenes, beats with cue words, on-screen strings, sourcing (tex/bib lines) | Code |
| **P1 Design** | Frontier model with visual judgment | One **design spec** per scene, in the scene grammar: template, element list and roles, hierarchy, reveal order. Stage renders it as a **wireframe** | Pixel coordinates, production code |
| **Author review of wireframes** | Author | Approve, or annotate with select-and-annotate | – |
| **P2 Implementation** | Cheaper coder, later a local model | Fills the approved spec: final strings, drawings, equation tokens, cue bindings. Layout comes from the grammar | New compositions, layout invention |
| **P3 Verify and repair** | Deterministic checks, OCR and pixel sweep, then the VLM critic on flagged stills | A defect list | – |
| | The **same P2 session**, resumed | Fixes, at most 2 rounds, then escalate | Cold-starting a new session |
| **P4 Author review** | Author, in the Stage player | Notes and drag overrides, sent as one batch to the same P2 session | – |
| **P5 Render** | Render workers, split across machines | Final video for the approved minute | Full renders before approval |

Wireframes and cue stills are cheap to review. Full renders happen once per approved minute.

## 4. Dispatch rules

- **Resume, never cold-start, for follow-ups.** Record every session ID in the dispatch log
  (`Orchestration/dispatch-log.jsonl` in the vault) at launch. Resume flags:
  - `codex exec resume <id>`;
  - `cursor-agent --resume <chatId>` (use `--output-format json` to capture the ID);
  - `kimi -S <id>` / `kimi -c`;
  - `agy --conversation <id>` / `agy -c`.
- **Persistent checker.** One stronger model reviews across scenes and sequences in a single resumed
  session, so its checking context accumulates. Builders can be cheaper.
- **Plan dispatch before work.** Decide, before anything runs:
  - which session builds each package;
  - who fixes (the builder);
  - who checks;
  - what context each one keeps;
  - which files each one owns.
- **Batch fixes.** One resumed turn with the full defect list.
- **Parallel only on disjoint files.** Use a git worktree per branch, and give each worker its own
  findings file.
- **Checkpoint commit** before every worker run (book repo AGENTS.md, commit-before-destroy).
  Write the checkpoint hash and the author's dirty paths into the brief.
- **zsh does not word-split** `$VAR`. Build worker flag lists as arrays.
- **Rules change only with evidence.** A new rule needs a frame that shows the defect. Enforce
  composition rules at scene ends, never at every frame.

## 5. Model routing (evidence as of 2026-11-02)

Judged by the author from same-moment comparison sheets:

| Model | Design | Implementation | Notes |
|---|---|---|---|
| Codex (GPT-5.6 Sol high) | Strong | Strong | Best scenes in minutes 3–7 |
| Gemini 4 Argon high (agy) | Strong, over-writes | Strong | Adds filler and unsourced text unless told not to |
| Gemini 3.8 Flash (agy) | Thin | Adequate | Needs explicit multi-pass prompting; measure its cost against Argon |
| Grok 4.5 (Cursor) | Thin | Adequate | Grok 4.7 (up to xhigh) is untested |
| Composer 2.5 (Cursor) | Thin | Adequate | Not for design work |

**Hypothesis to test:** once layout comes from the grammar, thin designers become adequate
implementers.

**Quota notes:**
- Cursor's plan excludes GPT-5.6 Sol.
- agy no longer lists Gemini 3.1 Pro.
- Research and fact-checking go to Gemini 3.8 Flash.

## 6. Figma

Any MCP-capable agent (Claude Code, Codex CLI, Cursor) can drive Figma headlessly through the
Figma MCP server's `use_figma` tool. The earlier Figma-based picture loop was slow, and the
code-built scenes are much higher quality, so Figma is no longer in the loop.
- The design pass outputs a grammar spec plus a Stage wireframe.
- Hand edits happen in the Stage player (select-and-annotate, drag overrides).
- Figma stays as a design reference only.

## 7. Experiments queue (run only after the grammar and verify loop exist)

1. **Frontier design comparison on a new sequence** (the next 5 minutes, turns 63+). Use the same
   storyboard for every model.
   - **P1 design pass only,** for each of: Codex Sol high, Gemini 4 Argon high, Grok 4.7 xhigh,
     Kimi, and a Claude model where quota allows.
   - The author ranks the wireframes.
   - **Separate design from implementation:** one fixed cheap implementer implements every design.
   - **Measure:** tokens, wall-clock, verify-loop rounds, and author preference.
2. **Flash multi-pass versus Argon single-pass,** at the same quality bar: total tokens and time.
3. **Local models** (two M3 Ultras), see `LOCAL_COMPUTE_PLAN.md` in the book repo:
   - local P2 implementer against the hosted one;
   - local VLM critic agreement with the author's accepted and rejected frames.

## 8. Standing project rules that bind every pass

- On-screen equation and figure numbers come from `Paper/The_Original_Power.aux`, never from
  `\label` names.
- No antithesis in any on-screen text. Run `tools/check_antithesis.py`.
- Every claim on screen comes from the storyboard, the narration or a cited tex/bib line.
- People are drawn with `PersonMark` (head and shoulders).
- Equations use the shared colour registry (`stage/equations`), and each symbol pulses on its
  spoken word.
