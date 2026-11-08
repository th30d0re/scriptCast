export type EarshotId =
  | 'click/double'
  | 'click/snap'
  | 'click/soft'
  | 'click/thock'
  | 'click/tink'
  | 'fit/key.space/thock_space'
  | 'fit/key/lubed'
  | 'fit/key/medium'
  | 'fit/key/thock'
  | 'fit/snap/switch_flip'
  | 'fit/tick/domino'
  | 'glitch/burst'
  | 'glitch/chirp'
  | 'glitch/crush'
  | 'glitch/data'
  | 'glitch/dropout'
  | 'glitch/hit'
  | 'glitch/mid'
  | 'glitch/scramble'
  | 'glitch/stutter'
  | 'hover_tick/air'
  | 'hover_tick/detent'
  | 'hover_tick/high'
  | 'hover_tick/mid'
  | 'hover_tick/ping'
  | 'notify/m0'
  | 'notify/m1'
  | 'notify/m2'
  | 'notify/m3'
  | 'notify/m4'
  | 'notify/m5'
  | 'notify/mid_tine'
  | 'notify/mid_wood'
  | 'swish/blade'
  | 'swish/bright'
  | 'swish/cane'
  | 'swish/flick'
  | 'swish/soft'
  | 'swish/stick'
  | 'swish/zing'
  | 'whoosh_big/cape'
  | 'whoosh_big/deep'
  | 'whoosh_big/gust'
  | 'whoosh_big/jet'
  | 'whoosh_big/reverse'
  | 'whoosh_big/swell'
  | 'whoosh_big/thump'
  | 'whoosh_big/thump_airy'
  | 'whoosh_big/thump_deep'
  | 'whoosh_fast/crisp'
  | 'whoosh_fast/heavy'
  | 'whoosh_fast/reverse'
  | 'whoosh_fast/sleeve'
  | 'whoosh_fast/tight'
  | 'whoosh_fast/whip'
  | 'whoosh_fast/whistle';

export type SoundCue = {
  component: string;
  event: string;
  frameOffset: number;
  sound: EarshotId;
  gain_db: number;
  note: string;
};

/**
 * Sound cues for every animation event in the Argon bake-off primitives.
 * Gains are mixed between -18 dB and -28 dB so cues sit cleanly underneath narration.
 */
export const sounds: SoundCue[] = [
  // 1. DrawRect (default 28-frame entrance)
  {
    component: 'DrawRect',
    event: 'corner start',
    frameOffset: 0,
    sound: 'click/tink',
    gain_db: -22,
    note: 'Two opposite corners ignite simultaneously and begin tracing orthogonal edges.',
  },
  {
    component: 'DrawRect',
    event: 'notch deploy',
    frameOffset: 9,
    sound: 'hover_tick/detent',
    gain_db: -25,
    note: 'Horizontal edge crosses center and deploys the 5-tier kit notch.',
  },
  {
    component: 'DrawRect',
    event: 'lines meet',
    frameOffset: 18,
    sound: 'click/snap',
    gain_db: -20,
    note: 'Four growing perimeter edges meet and lock at the opposite corners.',
  },
  {
    component: 'DrawRect',
    event: 'fill flicker pulse',
    frameOffset: 19,
    sound: 'glitch/chirp',
    gain_db: -26,
    note: 'Initial CRT panel fill power-up pulse right after corner closure.',
  },
  {
    component: 'DrawRect',
    event: 'fill on',
    frameOffset: 25,
    sound: 'fit/key/lubed',
    gain_db: -24,
    note: 'Panel fill settles to steady luminance after overshoot.',
  },

  // 2. DrawLine (default 24-frame entrance)
  {
    component: 'DrawLine',
    event: 'head launch',
    frameOffset: 0,
    sound: 'swish/soft',
    gain_db: -23,
    note: 'Bright leading head launches from origin vertex with a soft trace swish.',
  },
  {
    component: 'DrawLine',
    event: 'vertex turn',
    frameOffset: 8,
    sound: 'hover_tick/mid',
    gain_db: -26,
    note: 'Leading head rounds an intermediate polyline junction node.',
  },
  {
    component: 'DrawLine',
    event: 'vertex turn 2',
    frameOffset: 14,
    sound: 'hover_tick/high',
    gain_db: -26,
    note: 'Leading head passes second schematic waypoint.',
  },
  {
    component: 'DrawLine',
    event: 'terminal lock',
    frameOffset: 21,
    sound: 'hover_tick/ping',
    gain_db: -22,
    note: 'Leading head reaches terminal endpoint and wake settles.',
  },

  // 3. Flicker (Power-On, default 18 frames, and Power-Down, default 18 frames)
  {
    component: 'Flicker',
    event: 'ignition pulse 1',
    frameOffset: 1,
    sound: 'click/soft',
    gain_db: -23,
    note: 'First uneven relay strike as child element sparks on.',
  },
  {
    component: 'Flicker',
    event: 'capacitor dip',
    frameOffset: 4,
    sound: 'glitch/dropout',
    gain_db: -27,
    note: 'Brief primary dropout between first and second power pulses.',
  },
  {
    component: 'Flicker',
    event: 'catch pulse 2',
    frameOffset: 6,
    sound: 'glitch/stutter',
    gain_db: -25,
    note: 'Secondary catch pulse as voltage rail recovers.',
  },
  {
    component: 'Flicker',
    event: 'brightness overshoot',
    frameOffset: 11,
    sound: 'fit/snap/switch_flip',
    gain_db: -21,
    note: 'Phosphor brightness overshoot peak as rail locks on.',
  },
  {
    component: 'Flicker',
    event: 'steady lock',
    frameOffset: 16,
    sound: 'hover_tick/air',
    gain_db: -27,
    note: 'Overshoot decays into steady operational brightness.',
  },
  {
    component: 'Flicker:powerDown',
    event: 'rail unlatch',
    frameOffset: 0,
    sound: 'fit/key/thock',
    gain_db: -23,
    note: 'Power-down begins with initial voltage sag.',
  },
  {
    component: 'Flicker:powerDown',
    event: 'inductive flash',
    frameOffset: 5,
    sound: 'glitch/chirp',
    gain_db: -25,
    note: 'Brief inductive flyback flash before CRT collapse.',
  },
  {
    component: 'Flicker:powerDown',
    event: 'ember extinguish',
    frameOffset: 12,
    sound: 'click/soft',
    gain_db: -27,
    note: 'Final weak ember pulse fades to dark.',
  },

  // 4. FitBox (default 30-frame entrance)
  {
    component: 'FitBox',
    event: 'corner start',
    frameOffset: 0,
    sound: 'click/tink',
    gain_db: -23,
    note: 'Measured text box begins drawing perimeter from opposite corners.',
  },
  {
    component: 'FitBox',
    event: 'text flicker pulse',
    frameOffset: 11,
    sound: 'glitch/data',
    gain_db: -25,
    note: 'Measured typography flickers on inside the closing border.',
  },
  {
    component: 'FitBox',
    event: 'lines meet',
    frameOffset: 15,
    sound: 'click/snap',
    gain_db: -21,
    note: 'FitBox border closes around the measured text padding.',
  },
  {
    component: 'FitBox',
    event: 'text reveal',
    frameOffset: 22,
    sound: 'notify/mid_wood',
    gain_db: -24,
    note: 'Text and accent bar lock at steady brightness.',
  },

  // 5. StickFigure (default 22-frame draw-on)
  {
    component: 'StickFigure',
    event: 'head trace',
    frameOffset: 0,
    sound: 'hover_tick/mid',
    gain_db: -25,
    note: 'Head ring traces circular arc from top.',
  },
  {
    component: 'StickFigure',
    event: 'torso spine',
    frameOffset: 5,
    sound: 'swish/stick',
    gain_db: -25,
    note: 'Torso spine draws downward from collar to hip.',
  },
  {
    component: 'StickFigure',
    event: 'arms extend',
    frameOffset: 10,
    sound: 'fit/tick/domino',
    gain_db: -24,
    note: 'Both arms extend simultaneously from shoulder joints.',
  },
  {
    component: 'StickFigure',
    event: 'legs stance lock',
    frameOffset: 16,
    sound: 'click/thock',
    gain_db: -23,
    note: 'Both legs plant onto ground bracket and settle.',
  },

  // 6. BootIn variations ('snap', 'scan', 'glitch', default 36 frames)
  {
    component: 'BootIn:snap',
    event: 'corner start',
    frameOffset: 0,
    sound: 'click/tink',
    gain_db: -23,
    note: 'Snap preset starts fast DrawRect perimeter.',
  },
  {
    component: 'BootIn:snap',
    event: 'reticle snap lock',
    frameOffset: 15,
    sound: 'click/double',
    gain_db: -19,
    note: 'Border closes and spring-loaded content snaps into inner reticle.',
  },
  {
    component: 'BootIn:scan',
    event: 'notch & border start',
    frameOffset: 0,
    sound: 'hover_tick/detent',
    gain_db: -24,
    note: 'Scan preset starts notched DrawRect border.',
  },
  {
    component: 'BootIn:scan',
    event: 'raster sweep',
    frameOffset: 12,
    sound: 'whoosh_fast/tight',
    gain_db: -22,
    note: 'Horizontal scanline beam sweeps top-to-bottom unmasking child content.',
  },
  {
    component: 'BootIn:scan',
    event: 'scan complete',
    frameOffset: 30,
    sound: 'notify/mid_tine',
    gain_db: -24,
    note: 'Raster sweep completes and locks panel.',
  },
  {
    component: 'BootIn:glitch',
    event: 'bus sync start',
    frameOffset: 0,
    sound: 'glitch/chirp',
    gain_db: -24,
    note: 'Glitch preset initiates opposite-corner DrawRect.',
  },
  {
    component: 'BootIn:glitch',
    event: 'slice scramble',
    frameOffset: 12,
    sound: 'glitch/scramble',
    gain_db: -22,
    note: 'Deterministic horizontal slice displacement during high-intensity flicker.',
  },
  {
    component: 'BootIn:glitch',
    event: 'frame lock',
    frameOffset: 29,
    sound: 'fit/snap/switch_flip',
    gain_db: -20,
    note: 'Glitch bands snap into dead-center registration.',
  },
];

export default sounds;
