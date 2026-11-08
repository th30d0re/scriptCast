// Sound cues for every animation event, for rendering audio with earshot.
// Each generator reads the same phase tables and seeded flicker schedules the components use, so a cue lands on
// the exact frame its event draws. `frameOffset` is relative to the animation's start frame.
// Levels sit well under narration: structural events around -20 dB, texture (pulses, keys) -28 to -34 dB.
import {DRAW_LINE_DEFAULT_FRAMES, DRAW_LINE_PHASES, drawLineVertexFrames} from './DrawLine.opus';
import {DRAW_RECT_DEFAULT_FRAMES, DRAW_RECT_PHASES} from './DrawRect.opus';
import {FIT_BOX_DEFAULT_FRAMES, FIT_BOX_PHASES} from './FitBox.opus';
import {FLICKER_DEFAULT_FRAMES, POWER_DOWN_DEFAULT_FRAMES} from './Flicker.opus';
import {STICK_DEFAULT_FRAMES, STICK_PHASES} from './StickFigure';
import {BOOT_IN, type BootVariant} from './BootIn.opus';
import {flickerSchedule, powerDownSchedule, type Point} from './core';

export type EarshotId =
  | 'click/double' | 'click/snap' | 'click/soft' | 'click/thock' | 'click/tink'
  | 'fit/key.space/thock_space' | 'fit/key/lubed' | 'fit/key/medium' | 'fit/key/thock' | 'fit/snap/switch_flip' | 'fit/tick/domino'
  | 'glitch/burst' | 'glitch/chirp' | 'glitch/crush' | 'glitch/data' | 'glitch/dropout' | 'glitch/hit' | 'glitch/mid' | 'glitch/scramble' | 'glitch/stutter'
  | 'hover_tick/air' | 'hover_tick/detent' | 'hover_tick/high' | 'hover_tick/mid' | 'hover_tick/ping'
  | 'notify/m0' | 'notify/m1' | 'notify/m2' | 'notify/m3' | 'notify/m4' | 'notify/m5' | 'notify/mid_tine' | 'notify/mid_wood'
  | 'swish/blade' | 'swish/bright' | 'swish/cane' | 'swish/flick' | 'swish/soft' | 'swish/stick' | 'swish/zing'
  | 'whoosh_big/cape' | 'whoosh_big/deep' | 'whoosh_big/gust' | 'whoosh_big/jet' | 'whoosh_big/reverse' | 'whoosh_big/swell' | 'whoosh_big/thump' | 'whoosh_big/thump_airy' | 'whoosh_big/thump_deep'
  | 'whoosh_fast/crisp' | 'whoosh_fast/heavy' | 'whoosh_fast/reverse' | 'whoosh_fast/sleeve' | 'whoosh_fast/tight' | 'whoosh_fast/whip' | 'whoosh_fast/whistle';

export type SoundCue = {
  component:string;
  event:string;
  /** Frames after the animation's start (30 fps). */
  frameOffset:number;
  sound:EarshotId;
  gain_db:number;
  note:string;
};

const at = (fraction:number, frames:number) => Math.round(fraction*frames);
const cue = (component:string, event:string, frameOffset:number, sound:EarshotId, gain_db:number, note:string):SoundCue =>
  ({component, event, frameOffset, sound, gain_db, note});

/** Seeded flicker pulses → soft high ticks, rising a little in level as the tube warms, then the ignition flip. */
export function flickerCues(component:string, {offset = 0, durationFrames = FLICKER_DEFAULT_FRAMES, seed = 1, intensity = 1, prefix = '', ignite = true}:{offset?:number; durationFrames?:number; seed?:number; intensity?:number; prefix?:string; ignite?:boolean} = {}) {
  const s = flickerSchedule(seed, durationFrames, intensity);
  const out = s.pulses.map((pulse, i) => cue(component, `${prefix}flicker pulse ${i + 1}`, offset + pulse.start, 'hover_tick/high', -34 + 3*i, 'Seeded pulse; one tick per visible pulse, never per frame.'));
  if (ignite) out.push(cue(component, `${prefix}ignite`, offset + s.ignite, 'fit/snap/switch_flip', -22, 'Overshoot frame: the moment it powers on.'));
  if (ignite && s.sag >= 0) out.push(cue(component, `${prefix}sag`, offset + s.sag, 'hover_tick/air', -34, 'Single-frame sag after ignition; barely there.'));
  return out;
}

export function drawRectCues({offset = 0, durationFrames = DRAW_RECT_DEFAULT_FRAMES, seed = 7, intensity = 1, notch = false, component = 'DrawRect'}:{offset?:number; durationFrames?:number; seed?:number; intensity?:number; notch?:boolean; component?:string} = {}) {
  const P = DRAW_RECT_PHASES, d = durationFrames;
  return [
    cue(component, 'corner start', offset, 'click/soft', -24, 'Both start-corner brackets light together.'),
    cue(component, 'edges run', offset + at(P.edges[0], d), 'swish/soft', -30, 'Four edges travel; keep it a breath, not a whoosh.'),
    cue(component, 'lines meet', offset + at(P.meet, d), 'click/snap', -20, 'Edges close at the far corners; the frame flashes once.'),
    ...(notch ? [cue(component, 'notch', offset + at(P.notch[0] + .1, d), 'hover_tick/detent', -28, 'Notch V dips and its bars stack.')] : []),
    // A fill pulse within a frame of the meet would only muddy the snap, so it is dropped.
    ...flickerCues(component, {offset:offset + at(P.fill[0], d), durationFrames:(P.fill[1] - P.fill[0])*d, seed, intensity, prefix:'fill '})
      .filter(c => !(c.sound === 'hover_tick/high' && Math.abs(c.frameOffset - (offset + at(P.meet, d))) <= 1)),
  ];
}

export function drawLineCues({offset = 0, durationFrames = DRAW_LINE_DEFAULT_FRAMES, points, end = 'none', component = 'DrawLine'}:{offset?:number; durationFrames?:number; points:readonly Point[]; end?:'none'|'arrow'|'terminal'; component?:string}) {
  const P = DRAW_LINE_PHASES, d = durationFrames;
  return [
    cue(component, 'origin on', offset, 'hover_tick/mid', -28, 'Start terminal flickers on.'),
    cue(component, 'head leaves', offset + at(P.draw[0], d), 'swish/flick', -30, 'Head starts travelling.'),
    ...drawLineVertexFrames(points, d).map((f, i) => cue(component, `vertex ${i + 1}`, offset + f, 'hover_tick/detent', -28, 'Head passes a polyline corner; the vertex tick flares.')),
    cue(component, 'arrive', offset + at(P.draw[1], d), end === 'none' ? 'click/soft' : 'click/tink', -22, end === 'none' ? 'Head lands and begins to cool.' : `Head lands; ${end} snaps on.`),
  ];
}

export function powerDownCues({offset = 0, durationFrames = POWER_DOWN_DEFAULT_FRAMES, seed = 1, component = 'PowerDown'}:{offset?:number; durationFrames?:number; seed?:number; component?:string} = {}) {
  const s = powerDownSchedule(seed, durationFrames);
  return [
    cue(component, 'flare', offset + s.flare, 'hover_tick/ping', -30, 'Brief brightness flare before the cut.'),
    cue(component, 'dip 1', offset + s.dipA, 'glitch/dropout', -32, 'First dip.'),
    cue(component, 'dip 2', offset + s.dipB, 'glitch/dropout', -30, 'Second, deeper dip.'),
    cue(component, 'collapse', offset + s.collapse, 'whoosh_fast/reverse', -28, 'CRT squeeze to a line and pinch to a point.'),
    cue(component, 'off', offset + s.duration - 1, 'click/soft', -30, 'Last light out.'),
  ];
}

export function fitBoxCues({offset = 0, durationFrames = FIT_BOX_DEFAULT_FRAMES, seed = 11, intensity = 1, chars, tag = false, notch = false, keyEvery = 3}:{offset?:number; durationFrames?:number; seed?:number; intensity?:number; chars:number; tag?:boolean; notch?:boolean; keyEvery?:number}) {
  const P = FIT_BOX_PHASES, d = durationFrames;
  const textStart = P.text[0]*d, textFrames = (P.text[1] - P.text[0])*d;
  // A key tick every few characters, alternating two switches so the run does not sound like a machine gun.
  const keys:SoundCue[] = [];
  for (let k = 0, n = 0; k < chars; k += keyEvery, n++) {
    keys.push(cue('FitBox', `key ${n + 1}`, offset + Math.round(textStart + textFrames*k/chars), n % 2 ? 'fit/key/medium' : 'fit/key/lubed', -32, `Typing reaches character ${k + 1}.`));
  }
  return [
    ...drawRectCues({offset, durationFrames:(P.rect[1] - P.rect[0])*d, seed, intensity, notch, component:'FitBox'}),
    ...(tag ? [cue('FitBox', 'tag on', offset + at(P.tag[0], d), 'hover_tick/mid', -30, 'Mono tag flickers into the top edge.')] : []),
    ...keys,
    cue('FitBox', 'cursor out', offset + at(P.cursorOut[1], d) - 1, 'click/tink', -30, 'Cursor blinks out; text is settled.'),
  ];
}

export function stickFigureCues({offset = 0, durationFrames = STICK_DEFAULT_FRAMES}:{offset?:number; durationFrames?:number} = {}) {
  const P = STICK_PHASES, d = durationFrames;
  return [
    cue('StickFigure', 'head', offset, 'hover_tick/air', -32, 'Head circle starts drawing.'),
    cue('StickFigure', 'limbs land', offset + at(P.legs[1], d), 'hover_tick/mid', -30, 'Last leg lands; brief warm bloom. In a row, cue only every other figure.'),
  ];
}

export function bootInCues(variant:BootVariant, {offset = 0, durationFrames = BOOT_IN[variant].frames, seed = 21, intensity = 1, notch = false}:{offset?:number; durationFrames?:number; seed?:number; intensity?:number; notch?:boolean} = {}) {
  const spec = BOOT_IN[variant], d = durationFrames, component = `BootIn.${variant}`;
  const rect = drawRectCues({offset, durationFrames:(spec.rect[1] - spec.rect[0])*d, seed, intensity, notch, component});
  const c0 = offset + at(spec.content[0], d), c1 = offset + at(spec.content[1], d);
  if (variant === 'snap') return [
    // Snap stays light: drop the rect's travel swish and fill texture.
    ...rect.filter(c => c.event === 'corner start' || c.event === 'lines meet'),
    ...flickerCues(component, {offset:c0, durationFrames:(spec.content[1] - spec.content[0])*d, seed, intensity:.4*intensity, prefix:'content '}).filter(c => c.event !== 'content sag'),
  ];
  if (variant === 'scan') return [
    ...rect,
    cue(component, 'scan sweep', c0, 'whoosh_fast/tight', -28, 'Raster line starts down the box.'),
    cue(component, 'scan lands', c1, 'click/soft', -24, 'Wipe reaches the bottom edge; texture fades.'),
  ];
  return [
    ...rect,
    cue(component, 'tear', c0, 'glitch/stutter', -28, 'Content tears in bands.'),
    cue(component, 'tear 2', c0 + Math.round((c1 - c0)*.45), 'glitch/data', -32, 'Smaller second tear as the offsets decay.'),
    cue(component, 'lock', c1, 'click/thock', -22, 'Bands lock; content steady.'),
  ];
}

/** Default-parameter cue table for every component and event. */
export const sounds:SoundCue[] = [
  ...drawRectCues({notch:true}),
  ...drawLineCues({points:[[0, 0], [240, 0], [240, 120], [480, 120]], end:'arrow'}),
  ...flickerCues('Flicker'),
  ...powerDownCues(),
  ...fitBoxCues({chars:24, tag:true}),
  ...stickFigureCues(),
  ...bootInCues('snap'),
  ...bootInCues('scan'),
  ...bootInCues('glitch'),
];
