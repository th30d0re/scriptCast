# Worker sessions (resumable)

Resume these for follow-ups in the same area instead of starting a new agent:
- Codex: `codex exec resume <id> "<prompt>"`
- Cursor: `cursor-agent -p --resume <chatId> "<prompt>"`

| Workstream | Worker / model | Session id | Last used | Notes |
|---|---|---|---|---|
| Clip read-along + version in Settings | Cursor auto | 39867ff4-fe7c-4fa0-9d10-7735fcf63012 | 2026-10-04 | clip_transcribe.py, clip_transcripts.rs, settings.rs |
| Find & replace (Phase I) | Codex gpt-6-astra | 01a108e5-96cf-7753-a4da-774b8ef218df | 2026-10-04 | Ran out of quota; finished by Cursor |
| Screen 03 Figma alignment (Phase F) | Codex gpt-6-astra | 01a10797-a7ed-7030-8d78-b239a6972da5 | 2026-10-04 | table/read-along/editing context |
| Read-along (Phase E) | Codex gpt-6-astra | 01a10784-d34a-76c2-a947-170d07c22752 | 2026-10-04 | word_timings.py, read_along.rs |
| Editing UI (Phase D2) | Codex gpt-6-astra | 01a10467-3f77-7180-b0c1-868156fec170 | 2026-10-03 | editing.rs |
| Editing engine (Phase D1) | Codex gpt-6-astra | 01a1045c-c654-7452-8cfd-8ef49cad1e2b | 2026-10-03 | script.rs, export JSON modes |
| Job runner (Phase C, repair) | Codex gpt-6-astra | 01a10347-d5c9-7da1-8839-eae724aac5c4 | 2026-10-03 | runner/checkpoint/jobs |

Cursor runs so far did not record chat ids. From now on, every Cursor dispatch starts with `cursor-agent create-chat` and passes `--resume <id>`.
