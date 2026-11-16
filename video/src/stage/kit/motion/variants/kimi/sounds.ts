export type SoundCue = {
  component: 'DrawRect' | 'DrawLine' | 'Flicker' | 'PowerDown' | 'FitBox' | 'StickFigure' | 'BootIn/snap' | 'BootIn/scan' | 'BootIn/glitch';
  event: string;
  frameOffset: number;
  sound: string;
  gain_db: number;
  note: string;
};

export const sounds: readonly SoundCue[] = [
  {component: 'DrawRect', event: 'opposite corners ignite', frameOffset: 0, sound: 'click/soft', gain_db: -28, note: 'One cue for both corners; dry and quiet.'},
  {component: 'DrawRect', event: 'four edges meet', frameOffset: 22, sound: 'click/tink', gain_db: -25, note: 'Default 32-frame timing; scale with line completion.'},
  {component: 'DrawRect', event: 'fill on', frameOffset: 23, sound: 'hover_tick/high', gain_db: -31, note: 'A light electrical tick as the wash catches.'},
  {component: 'DrawRect', event: 'fill pulse', frameOffset: 25, sound: 'hover_tick/air', gain_db: -36, note: 'First restrained fill pulse; omit if narration is dense.'},
  {component: 'DrawRect', event: 'fill dropout', frameOffset: 27, sound: 'click/soft', gain_db: -36, note: 'Single low pulse before the wash stabilizes.'},
  {component: 'DrawRect', event: 'fill settled', frameOffset: 32, sound: 'notify/mid_tine', gain_db: -34, note: 'Optional; omit when a narration word lands here.'},
  {component: 'DrawLine', event: 'head starts', frameOffset: 0, sound: 'swish/soft', gain_db: -30, note: 'Short movement cue under the leading head.'},
  {component: 'DrawLine', event: 'polyline vertex', frameOffset: 8, sound: 'hover_tick/detent', gain_db: -33, note: 'Move to each measured vertex; omit for straight paths.'},
  {component: 'DrawLine', event: 'path complete', frameOffset: 26, sound: 'click/tink', gain_db: -27, note: 'Default 26-frame timing; a compact connection lock.'},
  {component: 'Flicker', event: 'first ignition', frameOffset: 0, sound: 'click/soft', gain_db: -31, note: 'Start of the seeded power-on sequence.'},
  {component: 'Flicker', event: 'pulse one', frameOffset: 2, sound: 'hover_tick/air', gain_db: -34, note: 'First uneven on-pulse.'},
  {component: 'Flicker', event: 'pulse two', frameOffset: 5, sound: 'click/soft', gain_db: -35, note: 'Low second pulse, not static.'},
  {component: 'Flicker', event: 'pulse three', frameOffset: 8, sound: 'hover_tick/mid', gain_db: -34, note: 'Final ignition pulse before overshoot.'},
  {component: 'Flicker', event: 'steady output', frameOffset: 14, sound: 'notify/m0', gain_db: -35, note: 'Very soft stable-state tine; optional under narration.'},
  {component: 'PowerDown', event: 'power cut', frameOffset: 0, sound: 'fit/snap/switch_flip', gain_db: -29, note: 'Matching reverse sequence begins.'},
  {component: 'PowerDown', event: 'last dropout', frameOffset: 13, sound: 'glitch/dropout', gain_db: -35, note: 'Short and restrained; no harsh noise tail.'},
  {component: 'PowerDown', event: 'fully off', frameOffset: 18, sound: 'click/soft', gain_db: -36, note: 'Optional near-silent relay release.'},
  {component: 'FitBox', event: 'border corners start', frameOffset: 0, sound: 'click/soft', gain_db: -30, note: 'Default 42-frame composite timing.'},
  {component: 'FitBox', event: 'border closed', frameOffset: 17, sound: 'click/tink', gain_db: -27, note: 'Measured text box is now established.'},
  {component: 'FitBox', event: 'box fill on', frameOffset: 18, sound: 'hover_tick/high', gain_db: -33, note: 'Keep below the border lock cue.'},
  {component: 'FitBox', event: 'text reveal', frameOffset: 28, sound: 'fit/key/lubed', gain_db: -29, note: 'Single soft key, not one cue per character.'},
  {component: 'FitBox', event: 'text settled', frameOffset: 42, sound: 'notify/mid_wood', gain_db: -36, note: 'Optional completion cue.'},
  {component: 'StickFigure', event: 'head draw', frameOffset: 0, sound: 'hover_tick/air', gain_db: -34, note: 'Start of optional draw-on; static replacement is silent.'},
  {component: 'StickFigure', event: 'torso draw', frameOffset: 5, sound: 'fit/tick/domino', gain_db: -35, note: 'Quiet body landmark.'},
  {component: 'StickFigure', event: 'arms draw', frameOffset: 10, sound: 'hover_tick/detent', gain_db: -35, note: 'One cue for both arms.'},
  {component: 'StickFigure', event: 'legs draw', frameOffset: 16, sound: 'swish/flick', gain_db: -35, note: 'One cue for both legs.'},
  {component: 'StickFigure', event: 'figure settled', frameOffset: 26, sound: 'click/tink', gain_db: -36, note: 'Optional for a single featured figure; omit for rows.'},
  {component: 'BootIn/snap', event: 'snap border start', frameOffset: 0, sound: 'click/snap', gain_db: -29, note: 'Preset-level cue; do not stack nested DrawRect cues.'},
  {component: 'BootIn/snap', event: 'snap border closed', frameOffset: 8, sound: 'click/snap', gain_db: -28, note: 'Fast mechanical close.'},
  {component: 'BootIn/snap', event: 'snap content reveal', frameOffset: 17, sound: 'fit/key/thock', gain_db: -29, note: 'Compact content lock-in.'},
  {component: 'BootIn/scan', event: 'scan border start', frameOffset: 0, sound: 'click/soft', gain_db: -31, note: 'Preset-level cue; do not stack nested DrawRect cues.'},
  {component: 'BootIn/scan', event: 'scan border closed', frameOffset: 14, sound: 'click/tink', gain_db: -29, note: 'Frame ready for vertical scan.'},
  {component: 'BootIn/scan', event: 'scan sweep', frameOffset: 24, sound: 'swish/cane', gain_db: -32, note: 'Soft sweep following the scan head.'},
  {component: 'BootIn/scan', event: 'scan settled', frameOffset: 44, sound: 'notify/m1', gain_db: -35, note: 'Optional stable-state cue.'},
  {component: 'BootIn/glitch', event: 'glitch border start', frameOffset: 0, sound: 'click/thock', gain_db: -30, note: 'Preset-level cue; do not stack nested DrawRect cues.'},
  {component: 'BootIn/glitch', event: 'glitch border closed', frameOffset: 11, sound: 'glitch/chirp', gain_db: -32, note: 'One short digital chirp.'},
  {component: 'BootIn/glitch', event: 'glitch content reveal', frameOffset: 20, sound: 'glitch/stutter', gain_db: -34, note: 'Single restrained stutter, never a noise loop.'},
  {component: 'BootIn/glitch', event: 'glitch settled', frameOffset: 40, sound: 'notify/mid_tine', gain_db: -35, note: 'Clean resolution after the disturbance.'},
];

export default sounds;
