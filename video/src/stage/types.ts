export type Word = {text: string; start_ms: number; end_ms: number};
export type Shot = {shot_id: string; turn_id: string; turn_index: number; start_ms: number; end_ms: number; props: Record<string, unknown>};
export type Segment = {instance: string; scene: string; start_ms: number; end_ms: number; shots: Shot[]};
export type StagePlan = {episode: string; fps: number; width: number; height: number; duration_ms: number; range: {start_ms: number; end_ms: number}; audio: string; cues: Record<string, number>; segments: Segment[]; words?: Record<string, Word[]>; warnings: string[]; episode_dir?: string};
