# R2 findings: fresh start vs resume in the dispatch data (step 17)

Measured 2026-11-06/07 (UTC) by `experiments/r2_measure/measure.py` (stdlib only; `python3 experiments/r2_measure/measure.py`, `--json` for the full structure). Snapshot: `sessions.jsonl` last record 2026-11-07T02:46:16Z.

## Inputs and sample sizes

| Source | Records | Notes |
|---|---|---|
| `workers/sessions.jsonl` (registry era, 2026-11-06/07) | 47 dispatches + 23 registry events | dispatcher's own log dir (`SCRIPTCAST_SESSION_LOGS`) |
| vault `Orchestration/dispatch-log.jsonl` (pre-registry, 2026-09-18 → 11-02) | 23 dispatches | older schema, no session ids |
| registries `*/stage/.router/sessions.json` | 1 file, 12 live + 5 archived rows | only `/Users/emmanuel/Dev/Tools/briefs/…` exists; the YT02 episode has no registry yet |

Classification is flags-based (`exec resume` / `-S` / `--resume` / `--conversation` per agent); brief text is never inspected, so tasks named "resume …" still count as starts. Result: **69 fresh starts, 1 resume dispatch** (the vault L4 record, a folded start + fix-round with `iterations=2`), and **6 `registry_resume` events with no matching dispatch record**. Every registry-era dispatch is a start.

## Wall time, exit status, acceptance (registry era, all starts)

| agent / kind | n | wall_s (med, mean, range) | verdict | wrote_files | tokens reported |
|---|---|---|---|---|---|
| codex / experiment | 20 | 104.8, 106.8, [65.9–206.0] | success 20 | 20/20 | 0/20 |
| agy / experiment | 10 | 203.3, 213.3, [82.8–360.8] | success 10 | 10/10 | 0/10 |
| codex / infra | 13 | 273.7, 1563.4, [5.0–7351.5] | success 12, failure 1 | 8/13 | 0/13 |
| kimi / infra | 3 | 1015.8, 711.3, [0.8–1117.4] | success 2, failure 1 | 0/3 | 0/3 |
| agy / infra | 1 | 71.9 | success | 0/1 | 0/1 |

`acceptance` is `not_run` on all 47 registry-era records. In the vault era, tokens are reported on 6 records (all codex implement starts: median 91,123, range 53,067–104,916), wall on 9, acceptance pass 4 / fail 4 / not_run 15. kimi's `wrote_files=false` on two successful runs is a measurement artifact: `wrote_files` diffs the dirty tree and misses work the worker committed itself.

## What the resume side shows

All 6 resumed turns exist only as registry events (the resumes were run without `--log`), so a resume has **no wall time, verdict, acceptance, or reported tokens** anywhere in the data. What the registry does record is injected text per turn (cumulative deltas; all `tokens_kind: proxy` = brief_chars/4):

| session | turns | start brief | resume injections (chars) | proxy tokens |
|---|---|---|---|---|
| briefs/boxes/builder | 6 | 2,996 chars | +862, +1269, +597, +995, +950 | 749 → 1,917 |
| briefs/r3_critic/builder | 2 | 2,618 chars | +1,091 (session ended `dead`) | 654 → 927 |

Resume turns inject 20–42% of their start brief's size as new text (boxes mean 31%). This is the only token-shaped signal available, and it is a proxy for *injected* text: a resumed worker re-processes its accumulated context server-side, so the proxy understates resume cost and ignores caching entirely. It cannot answer "does resume save tokens".

## Follow-ups dispatched as fresh starts

Five tasks were followed up with a cold start instead of a resume; the superseded starts burned **17,772 s ≈ 4.9 worker-hours** in one evening:

| task | starts | wasted wall on superseded start | redispatch |
|---|---|---|---|
| r3_followup2 | 2 | 7,351 s (exit 0, wrote nothing) | 134 s, wrote files |
| dispatch_fix | 2 | 7,351 s (exit 0, wrote nothing) | 274 s, wrote files |
| ui_merge | 2 | 3,064 s (exit 0, wrote nothing) | 215 s, wrote nothing |
| r3_critic | 2 | 5 s (gpt-6.1-terra exit 1) | 329 s on gpt-5.6-terra, wrote files |
| e3_svg | 2 | 0.8 s (k3 exit 1) | 1,117 s on kimi-code/k3, success |

The three 22:16:12 starts ran concurrently for ~1–2 h each, wrote nothing, and were redispatched fresh within five minutes.

## Data-quality hazards found

1. **Session-id collisions.** codex id `01a126f5…` is recorded for 4 different tasks across 3 worktrees; agy id `76d3233c…` for 2 experiment cells. Joins by `session_id` are unreliable for codex/agy; registry key + timestamp is the reliable join. `capture_session_id` for codex needs a fix.
2. **Tokens unreported in the registry era.** `capture_session_id` parses usage only from cursor-agent's JSON output, and no cursor-agent dispatch ran; 47/47 records have `tokens: 0`.
3. **Resumes skip the dispatch log.** `cmd_resume` logs only when `--log` is passed; the 6 observed resumes were not, which is why the resume side has no outcome data.
4. **Acceptance unset.** Registry-era acceptance is uniformly `not_run`; `verdict` is the CLI exit code and nothing more.

## Verdict on the step-17 question

**Too thin to conclude.** The planned measurement (three YT02 fixes × {current practice, registry + context}) was never run, and the data that exists cannot substitute for it: the resume sample is 2 sessions / 6 turns carrying only a text-injection proxy, and the 69-start fresh sample is token-blind. What the data does support:

- Follow-ups-as-cold-starts are common (5 of ~12 registry-era infra tasks) and expensive (4.9 worker-hours wasted in one evening, all recoverable by resume).
- Resumed turns carry roughly a third of a start brief's injected text (2 sessions, proxy metric only).
- The registry itself works: 17 rows, correct supersede archiving, turn counting consistent with the event stream.

## Before the A/B can measure anything

1. Always pass `--log` on resume (or default it), so resumed turns get wall/verdict records.
2. Fix codex session-id capture (collisions above); record per-call usage where the CLI exposes it.
3. Set `acceptance` on registry-era runs; exit code alone passes workers that wrote nothing.
4. Then run the planned 3-fixes-×-2-conditions experiment and record `experiments/router_eval/r2_tokens.csv` as the plan specifies.

## Step 17 A/B run (2026-11-07)

Three real YT02 fixes, each run from the commit its brief was written at, Codex gpt-5.6-luna. A = current-practice brief, fresh start. B = no registry session existed, so start with `--context` from the premiere notebook (d3f54f50). Acceptance = the episode's own `tools/check_*.mjs` passes and the worker changed files.

| fix | A brief chars | B brief chars | A wall s | B wall s | accepted A/B |
|---|---|---|---|---|---|
| minute-two figure centring | 2,880 | 4,233 | 344 | 296 | yes/yes |
| dashboard wage plane | 4,561 | 5,058 | 380 | 350 | yes/yes |
| first-minute dictionary gutter | 5,940 | 6,437 | 530 | 417 | yes/yes |

Context adds 500–1,350 chars to the brief and cut wall time 8–21% (mean 14%), with equal acceptance. n=3 per arm, one run each, so this is direction not proof. Tokens are still a brief-length proxy: Codex's reported usage is not captured. Condition B was never a true resume (no prior session per scene), so the registry's resume path remains unmeasured; the "70% token reduction" claim is untested. Diffs in `experiments/router_eval/r2_runs/`.

## True resume vs fresh start (2026-11-07)

One small follow-up per fix ("add a one-line comment naming the cue"), Codex gpt-5.6-luna, each arm in its own worktree with the original fix applied. Tokens are Codex's reported totals for that process (read from `~/.codex/sessions`; the counter restarts per process, so a resumed turn's total is that turn's cost). Data: `experiments/router_eval/r2_resume.csv`.

| fix | resume total / cached | fresh total / cached | resume wall | fresh wall |
|---|---|---|---|---|
| minute-two-figure-centering | 298,277 / 197,760 | 308,740 / 272,640 | 25 s | 51 s |
| dashboard-tray-wage-plane | 311,806 / 206,976 | 157,386 / 108,544 | 18 s | 32 s |
| first-minute-dictionary-gutter | 346,537 / 237,184 | 295,362 / 267,008 | 20 s | 52 s |

- Resume is about 2x faster on wall time.
- Resume is not cheaper in tokens: total 956,620 vs 761,488 fresh (+26%). Uncached input is worse still, about 100k per resumed turn vs 26-48k fresh, because replaying the prior history misses the cache.
- The plan's 70% token-reduction claim is not supported for small follow-ups. It may hold for large follow-ups where a fresh start must re-read much more; untested. n=3.
- Harness gap: stage_dispatch did not capture reported tokens for these runs (rows fell back to the brief proxy). The codextok parser needs to read the session file, not stdout.
