# Brief: HyperFrames vs ScriptCast comparison

READ-ONLY ANALYSIS. Do not modify, create, or delete any file except the one output file below.
Do not run git commands that change state. Do not install packages. Do not run network commands.

## Repos
- ScriptCast (owner's project): /Users/emmanuel/Dev/Tools/scriptCast
  Key areas: README.md, CHANGELOG.md, scriptcast/ (Python engine/tools), video/ (Remotion-style video),
  swift/, ui/SPEC.md, plans/Native_UI/, docs/briefs/ (especially FINDINGS-*.md, craft-video-pipeline.md).
- HyperFrames (heygen-com/hyperframes, cloned): /Users/emmanuel/Dev/Tools/hyperframes
  Start with README.md, AGENTS.md, DESIGN.md, packages/, skills/, registry/, examples/, docs/.
  Binary assets are LFS pointers; ignore them.

## Questions to answer
1. What each project is: purpose, architecture, render pipeline, input format, tech stack, license.
2. Overlap map: table of capabilities (script/narration, TTS, composition/timeline, rendering,
   templates/registry, agent/skill integration, CLI, preview, QA) with where each lives in each repo (file paths).
3. What ScriptCast could adopt from HyperFrames — ranked, each with: what it is, file paths in
   hyperframes, effort (S/M/L), and why it fits ScriptCast's workflow.
4. What ScriptCast has that HyperFrames lacks.
5. The strategic decision, with a clear recommendation and reasoning:
   (A) integrate ScriptCast into HyperFrames (ScriptCast becomes a plugin/frontend/skill on top of it),
   (B) integrate HyperFrames into ScriptCast (use it as a render backend/dependency),
   (C) cherry-pick ideas/code only.
   Address license compatibility, upstream churn risk, and what ScriptCast's existing video/ pipeline would become.
6. A concrete first step (1-2 weeks of work) for the recommended option.

Cite file paths for every factual claim. Mark anything inferred rather than read as (inferred).

## Output
Write the report to: /Users/emmanuel/Dev/Tools/scriptCast/docs/briefs/FINDINGS-hyperframes.md
