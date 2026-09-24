PROHIBITIONS (read first): Do not run git commit/add/checkout/stash or any history command. Do not edit anything under `/Users/emmanuel/Documents/Theory/scriptCast` (read only). Write ONLY inside `/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/cards_json/` and `/tmp`. Do not touch `video/public/episode/`, do not use `rm -rf`, do not call any network API, and never read, print or write any API key or environment secret. No image of a real person, ever.

# Task: design the data cards for the Chapter 135 rebuttal video

Context: a 13-minute vertical (1080x1920) reaction video for Instagram Reels opposing the MA
Question 9 "Yes" position (the narrator argues NO / repeal). Cards are React components driven
by JSON. The shot list says which cards exist and what each must show. The user reviewed the
first two rendered cards and wants **more detailed, more informative cards**, plus vector art
where a picture helps. Your job: produce the card JSON for every card shot, and a precise list
of SVG assets to generate.

## Read first

- Shot list: `/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/chapter135_rebuttal_shotlist.md` (shots G-01..G-16; note each shot's provenance tag and Card references)
- Citation deck (the ONLY source of facts/numbers): `/Users/emmanuel/Documents/Theory/TheOriginalPower/Paper/chapter135_rebuttal_video/citations.md`
- Script (what is being said while each card is on screen; anchors are in the shot list): `/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/podcasts/chapter135_rebuttal_reply.md`
- Reference cards (copy and style): `/Users/emmanuel/Documents/Theory/scriptCast/video/examples/g10.json`, `g11.json`, `g10_svg.json`
- Components and their props (the JSON schema): `/Users/emmanuel/Documents/Theory/scriptCast/video/src/cards/{TimelineCard,StatBarsCard,TitleCard,CompareCard,Frame}.tsx`, `/Users/emmanuel/Documents/Theory/scriptCast/video/src/safeZone.ts`, `theme.ts`
- Earlier AI-generated versions to learn intent from (do not copy their errors): `/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/images/gNN.png`

## Rules

1. **Facts only from citations.md.** Every number, date and quotation on a card must trace to a
   card in citations.md. Do not add statistics from memory. If a shot needs a number that is
   not in citations.md, leave it out and record it in `component_gaps.md` as "needs source".
   Known corrections that must hold: the emergency preamble came **69 days** after signing
   (not 71); reasons reported were "without delay" (AP) and "Day One implementation" (NBC
   Boston), not "public convenience"; Louisiana white registration fell 164,088 to 125,437
   (-24%), it did not "barely move". Cards state facts, not rhetoric: no insults, no
   characterizing motives.
2. **Provenance line.** Every card has `sources` in the form "Sources: A · B · C" naming real
   sources from the citation card. The renderer adds the QR placeholder itself.
3. **Layout budget.** The renderer lays cards inside a ~950 x 880 px safe box (top 14% and
   bottom 40% of the frame are covered by the platform UI). Fit as much detail as reads
   at phone size: minimum body text ~21px as rendered. Validate every card by rendering it
   (below) and looking at the PNG: nothing clipped, nothing cut off at the bottom of the box.
4. **Card types.** Use `TimelineCard`, `StatBarsCard`, `TitleCard`, `CompareCard` with their
   exact prop names. If a shot cannot be expressed well (for example a map, a silhouette
   comparison, a two-axis chart), pick the closest component, set `svgAsset` to a planned
   path (see assets below) and document the gap in `component_gaps.md` with what component
   change would fix it. Do NOT edit the components.
5. **svgAsset** is a path under `video/public/` such as `assets/quiver/g15_ar15_mini14.svg`.
   The art slot is a 64px-high contained strip in the current renderer (a known limit; note
   in gaps any card that needs a larger art area). Planned paths need not exist yet.
6. Shots that are not cards: G-01 (freeze-frame annotation of the real event wall) is not a
   card, skip it, and note it. G-04 and G-16 are simple: make them `TitleCard`s. G-02, G-03,
   G-05..G-09, G-12..G-15 are cards to design; G-10 and G-11 exist already (you may enrich
   them, keep all their currently required copy exactly).

## Output

In `/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/cards_json/`:

- `g02.json` ... one JSON file per card shot, named by lowercase shot id, each with a
  `component` field. Valid JSON, UTF-8, real minus sign (U+2212) for negatives.
- `assets.md`: a table of every SVG asset worth generating with Quiver Arrow 2, one row per
  asset: `path`, `used by shot`, `what it shows`, and a **ready-to-send Arrow prompt** written
  per the Arrow guidance (subject, style, exact palette hex codes from theme.ts, composition,
  any text verbatim, transparent/navy background, flat vector, no gradients unless asked),
  plus an `instructions` string. Include at least: the AR-15 versus Ruger Mini-14
  side-by-side silhouette for G-15 (two rifles drawn to the same scale, flat outline,
  neutral, no brand marks, the point being the same cartridge and similar function, different
  cosmetics; keep the facts from citations.md), and any others you judge useful (a simple
  Boston/neighbourhood map motif, a ballot-box/gavel icon set, a lead-pipe / redlining
  motif). No real people, no likeness of any public figure, no logos.
- `component_gaps.md`: every card whose design is compromised by the current components or
  by a missing sourced number, with a concrete proposal.
- `SUMMARY.md`: one line per card: shot id, component, what it shows, provenance cards used.

## Validate (required)

Render each card to check it, from anywhere:

```bash
/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/scriptcast-video still \
  /Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/cards_json/g05.json /tmp/g05.png
```

Look at the PNG. If text is clipped at the bottom, remove or shorten content until it fits.
`still` merges your JSON over the composition's default props, so a field you omit may fall
back to the default: set optional fields you do not want to `""` or `[]`. Report in your
final message: files written, which cards render without clipping, and the gaps.
