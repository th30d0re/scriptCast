// The JSON timeline that ScriptCast emits (scriptcast/timeline_export.py) and the
// Timeline composition renders. timeline.schema.json is the same contract.
export type TimelineItem = {
  card: string;
  props: Record<string, unknown>;
  startSec: number;
  endSec: number;
};
export type Timeline = {
  fps: number;
  width: number;
  height: number;
  durationSec: number;
  items: TimelineItem[];
};
