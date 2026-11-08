export interface SoundCue {
  component: string;
  event: string;
  frameOffset: number; // relative to the animation's start
  sound: string; // earshot id
  gain_db: number;
  note: string;
}

export const sounds: SoundCue[] = [
  // DrawRect
  {
    component: 'DrawRect',
    event: 'corner_start',
    frameOffset: 0,
    sound: 'click/snap',
    gain_db: -18,
    note: 'Crisp initial tactile strike as opposite corners initiate vector path tracing',
  },
  {
    component: 'DrawRect',
    event: 'trace_motion',
    frameOffset: 6,
    sound: 'swish/zing',
    gain_db: -24,
    note: 'Subtle high-frequency wire travel hum along borders',
  },
  {
    component: 'DrawRect',
    event: 'lines_meet',
    frameOffset: 18,
    sound: 'click/soft',
    gain_db: -16,
    note: 'Closing perimeter snap as orthogonal lines join at complementary corners',
  },
  {
    component: 'DrawRect',
    event: 'fill_on',
    frameOffset: 20,
    sound: 'whoosh_fast/crisp',
    gain_db: -22,
    note: 'Soft surface wash ignition and panel tint reveal',
  },

  // DrawLine
  {
    component: 'DrawLine',
    event: 'spark_ignite',
    frameOffset: 0,
    sound: 'hover_tick/ping',
    gain_db: -20,
    note: 'Leading head spark initiation ping',
  },
  {
    component: 'DrawLine',
    event: 'travel_whistle',
    frameOffset: 8,
    sound: 'whoosh_fast/whistle',
    gain_db: -26,
    note: 'Quiet aerodynamic lead head traveling along polyline',
  },
  {
    component: 'DrawLine',
    event: 'lead_arrive',
    frameOffset: 28,
    sound: 'click/tink',
    gain_db: -18,
    note: 'Terminal arrival tick as leading spark settles into endpoint node',
  },

  // Flicker
  {
    component: 'Flicker',
    event: 'ignition_pulse',
    frameOffset: 0,
    sound: 'glitch/chirp',
    gain_db: -20,
    note: 'CRT cathode tube filament warmup spark',
  },
  {
    component: 'Flicker',
    event: 'stutter_burst',
    frameOffset: 5,
    sound: 'glitch/stutter',
    gain_db: -19,
    note: 'Rapid micro-flicker brightness overshoot during power surge',
  },
  {
    component: 'Flicker',
    event: 'power_settle',
    frameOffset: 14,
    sound: 'notify/m0',
    gain_db: -24,
    note: 'Gentle resonant harmonic chime confirming steady-state voltage lock',
  },
  {
    component: 'Flicker',
    event: 'power_down',
    frameOffset: 0,
    sound: 'glitch/dropout',
    gain_db: -18,
    note: 'Sudden collapse of tube raster field and phosphor drain',
  },

  // FitBox
  {
    component: 'FitBox',
    event: 'chassis_draw',
    frameOffset: 0,
    sound: 'click/double',
    gain_db: -18,
    note: 'Twin corner trigger click for box outline expansion',
  },
  {
    component: 'FitBox',
    event: 'type_reveal',
    frameOffset: 14,
    sound: 'fit/key/thock',
    gain_db: -15,
    note: 'Deep mechanical switch engagement as measured text renders',
  },

  // StickFigure
  {
    component: 'StickFigure',
    event: 'head_node',
    frameOffset: 0,
    sound: 'hover_tick/high',
    gain_db: -22,
    note: 'Head circle glyph vector draw tick',
  },
  {
    component: 'StickFigure',
    event: 'torso_draw',
    frameOffset: 5,
    sound: 'hover_tick/mid',
    gain_db: -24,
    note: 'Vertical spine stroke descending',
  },
  {
    component: 'StickFigure',
    event: 'limbs_plant',
    frameOffset: 16,
    sound: 'fit/tick/domino',
    gain_db: -19,
    note: 'Bipedal arms & legs stroke completion lock',
  },

  // BootIn (named variations)
  {
    component: 'BootIn:snap',
    event: 'relay_click',
    frameOffset: 16,
    sound: 'fit/snap/switch_flip',
    gain_db: -14,
    note: 'Hard industrial mechanical toggle snap locking child element in place',
  },
  {
    component: 'BootIn:scan',
    event: 'crt_scan_beam',
    frameOffset: 10,
    sound: 'swish/bright',
    gain_db: -22,
    note: 'Luminous beam sweep downward through child payload',
  },
  {
    component: 'BootIn:glitch',
    event: 'raster_shift',
    frameOffset: 12,
    sound: 'glitch/burst',
    gain_db: -17,
    note: 'Horizontal sync pulse displacement slice with chromatic fringing',
  },
];
