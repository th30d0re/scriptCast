# Brief: scriptCast landing page (for Figma AI)

Paste everything below the line into Figma AI (Figma Make or First Draft). Fill the `[FILL]` fields first.

---

Design a single-page marketing site for **scriptCast**, desktop first with a responsive mobile layout. It'll be shown to judges of an AI product competition ([FILL: competition name]) and to working creators. The judges explicitly reject chat wrappers, so the page must not look like an AI chatbot product. No chat bubbles, no prompt box as the hero, no "Ask AI" buttons, no sparkle icons, no purple-to-pink gradients.

## What scriptCast is

A local, Apple Silicon toolchain that compiles a Markdown script into verified speech audio, frame-accurate video, and native Ableton Live / Logic Pro / Final Cut Pro sessions. AI voice and recognition models are components inside a deterministic pipeline: every turn is synthesized, checked against the script by on-device speech recognition, and re-synthesized automatically if it fails. Everything locks to one clock, so audio and video never drift.

Tagline options (pick one, or keep both for A/B):
- "Compile your script. Verify every word."
- "Production as compilation."

## Audience and tone

Engineers, technical judges, and independent video essayists and podcasters. The tone is precise, quiet and confident, closer to a developer tool or audio-plugin site than to a SaaS AI startup. Show real artifacts (code, waveforms, timelines) instead of illustrations of robots or brains.

## Visual language

Match the native app, which is dark:
- Background near-black navy (#14171C to #1A1F27), panels a step lighter (#20252E), hairline borders (#2C333D).
- Text off-white (#E6E9EE), muted text #8A93A0.
- Accent soft blue (#8FB4E8) for selection and waveforms. Status colors used sparingly: pass green, fail red, archive amber (#E8B04A).
- Type: a clean grotesk for headings (Inter or similar) and a monospace (JetBrains Mono or similar) for code, file names, and timestamps. Monospace is a big part of the identity.
- Motifs: waveform bars, a horizontal timeline with turn blocks, timestamps like `00:16.120`, hashes like `turn_id 9f2c…`.
- A lot of negative space. Square-ish corners (4–6 px radius). No glassmorphism, and no 3D blobs.

## Page sections, in order

1. **Nav:** wordmark "scriptCast" with a small waveform glyph. Links: How it works, Verification, Video, App, GitHub. Primary button "Download for macOS" (Apple Silicon), secondary "View on GitHub".

2. **Hero:** headline (the tagline), and a subhead in one sentence from "What scriptCast is". The visual is a split composition. On the left, a Markdown script snippet in monospace:
   ```
   Emmanuel Theodore (00:00)
   I was first in line for the iPhone Duo on launch day. [pause:400]
   ```
   An arrow leads to the right, where a waveform timeline shows turn blocks labelled 000, 001, 002 with green check badges. Below: "Apple Silicon · Runs offline · Open source".

3. **The pipeline:** a horizontal flow diagram of eight nodes: Markdown script → Per-speaker engine → Rust DSP → Manifest (one clock) → Verify → Repair (loops back to the engine) → Mix + captions → Exports. Make the Repair loop-back arrow visually prominent. Each node gets one short line of caption.

4. **"Not a wrapper": three columns**
   - *Compiled, not prompted.* Each turn has a content hash. Edit one sentence and only that turn re-renders. Approved audio stays byte-identical.
   - *Verified against the source.* On-device Whisper, plus a second recognizer and a phoneme stress check, score every turn against the script. Failures are re-synthesized on their own.
   - *One clock for sound and picture.* `episode_manifest.json` drives the mix, the video cut, captions, and DAW sessions.

5. **Verification spotlight:** a table mock in the app's style with columns Turn, Time, Transcript, Verification. One row is a failure ("live" heard as "lyve", red, stress mismatch) and the rows around it pass. A small callout reads "Repair pass 1 → passed".

6. **Engines strip:** a row of plain text labels, not logos: Kokoro · Chatterbox · OmniVoice · Phonon · Qwen3 Tone · ElevenLabs (optional cloud) · Archive clips. Caption: "Choose per speaker. Local by default."

7. **Video:** headline "Frame-accurate video from the same manifest." A 16:9 frame placeholder for an embedded demo video ([FILL: video URL]), beside a 9:16 Shorts placeholder. Spec chips: 3840×2160 · 1920×1080 · 1080×1920 · FCPXML 1.10 · SRT / WebVTT. Note: "Rendered stills are read back with OCR and pixel checks, the same way audio is."

8. **The app:** a large screenshot placeholder of the native macOS app (dark UI: a waveform timeline across the top, a turn table below, a side inspector). I'll drop in real screenshots. Three feature callouts around it: "60 Hz timeline and waveforms", "Record a live take into any turn", "Open in Live or Logic in one click".

9. **Exports:** three cards for Ableton Live (.als), Logic Pro, and Final Cut Pro (FCPXML). Use generic icons, not the vendors' logos.

10. **Built like a compiler: numbers row.** ~160k lines of Rust · 6 speech engines · 2 independent recognizers · 0 cloud calls required. Monospace numerals.

11. **Final call to action:** "Download scriptCast for macOS 0.22.0", plus "Read the architecture", which links to GitHub. A small note: "Apple Silicon required."

12. **Built on (credits):** a quiet section above the footer headed "Standing on open work". Show small monospace text chips in groups, with no logos:
    - Voices: Kokoro · Chatterbox · OmniVoice · Phonon · Qwen3-TTS
    - Listening: Whisper · Phonon-2 · wav2vec2 · Tesseract · SigLIP · MiniCPM-V
    - Platform: Apple MLX · FFmpeg · egui · soundcraft · Remotion
    - Ported: HyperFrames (HeyGen), ducking and spectral carve, rewritten in Rust

    End it with a link: "Full acknowledgements →" (to ACKNOWLEDGEMENTS.md on GitHub).

13. **Footer:** wordmark, GitHub link, license, "Made by [FILL: name / studio]", and a competition badge placeholder.

## Deliverables

- Desktop frame at 1440 px wide and mobile frame at 390 px wide.
- Use auto layout throughout. Make the turn row, the status badge (pass / fail / not verified), the spec chip, and the pipeline node reusable components.
- Use real copy from this brief, not lorem ipsum.
- Mark the image placeholders clearly (app screenshot, demo video, Shorts) so I can replace them.
