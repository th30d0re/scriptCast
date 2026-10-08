# Worker sessions (resumable)

Resume these for follow-ups in the same area instead of starting a new agent:
- Codex: `codex exec resume <id> "<prompt>"`
- Cursor: `cursor-agent -p --resume <chatId> "<prompt>"`

| Workstream | Worker / model | Session id | Last used | Notes |
|---|---|---|---|---|
| Phase L UI | Codex gpt-6-astra medium; Cursor auto | 01a11d3a-34bf-70d0-9395-31a37877f81b; a35e0385-09bb-48dc-adee-f21f73dedb32 | 2026-10-08 | worktree phase-l; merged |
| Phase L Python (scriptcast-project) | Cursor auto | ed3cdcae-5436-4f86-b4c9-d60a5968fa51 | 2026-10-08 | worktree phase-l-py; merged |
| Phase K UI (Codex start, Cursor finish + visual) | Codex gpt-6-astra medium; Cursor auto | 01a11c7f-d367-7220-8619-fe06e74c1b41; 8b3ab914-e69b-4d0d-bd04-48e2f5133cdc | 2026-10-08 | worktree phase-k; merged |
| Phase K Python (edit-json, tags-json, speakers-json) | Codex gpt-5.6-sol medium | 01a11c26-1ca7-7362-b0e4-95ab6658a302 | 2026-10-08 | worktree phase-k-py; merged |
| Phase J visual round 1 | Cursor auto | 5f23e600-cf85-40c0-a9b8-e5851ed44295 | 2026-10-07 | worktree phase-j; Codex quota hit until 2026-10-08 02:36 |
| Phase J UI (new episode, live render, detached jobs, pipeline) | Codex gpt-6-astra medium | 01a11920-93fe-7b81-a228-9c47e7123b2d | 2026-10-07 | worktree phase-j, branch ui/phase-j |
| Phase J Python (dry-run JSON, progress JSONL, --resume, --export-only) | Codex gpt-5.6-sol medium | 01a11920-93ff-7062-bf14-d7a89372270d | 2026-10-07 | worktree phase-j-py, branch py/phase-j |
| Clip read-along + version in Settings | Cursor auto | 39867ff4-fe7c-4fa0-9d10-7735fcf63012 | 2026-10-04 | clip_transcribe.py, clip_transcripts.rs, settings.rs |
| Find & replace (Phase I) | Codex gpt-6-astra | 01a108e5-96cf-7753-a4da-774b8ef218df | 2026-10-04 | Ran out of quota; finished by Cursor |
| Screen 03 Figma alignment (Phase F) | Codex gpt-6-astra | 01a10797-a7ed-7030-8d78-b239a6972da5 | 2026-10-04 | table/read-along/editing context |
| Read-along (Phase E) | Codex gpt-6-astra | 01a10784-d34a-76c2-a947-170d07c22752 | 2026-10-04 | word_timings.py, read_along.rs |
| Editing UI (Phase D2) | Codex gpt-6-astra | 01a10467-3f77-7180-b0c1-868156fec170 | 2026-10-03 | editing.rs |
| Editing engine (Phase D1) | Codex gpt-6-astra | 01a1045c-c654-7452-8cfd-8ef49cad1e2b | 2026-10-03 | script.rs, export JSON modes |
| Job runner (Phase C, repair) | Codex gpt-6-astra | 01a10347-d5c9-7da1-8839-eae724aac5c4 | 2026-10-03 | runner/checkpoint/jobs |

Cursor runs so far did not record chat ids. From now on, every Cursor dispatch starts with `cursor-agent create-chat` and passes `--resume <id>`.
