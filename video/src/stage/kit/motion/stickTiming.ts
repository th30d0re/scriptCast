/** Stick-figure timing, kept in a leaf module so the sound table can read it without importing the component. */
export const STICK_PHASES = {head:[0, .3], torso:[.22, .46], arms:[.4, .7], legs:[.6, .92], settle:[.92, 1]} as const;
export const STICK_DEFAULT_FRAMES = 20;
