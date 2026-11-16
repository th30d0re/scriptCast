#!/usr/bin/env node
/**
 * Render a Stage episode one plan segment (scene) at a time, then join the chunks.
 *
 *   STAGE_EPISODE_DIR=<stage dir> node scripts/render-episode-chunks.mjs <plan.json> <outDir> \
 *     [--browser-executable <chrome>] [--only Scene,Scene] [--until-ms N]
 *
 * Each segment renders to <outDir>/chunks/NN-<Scene>.mkv (h264 + PCM audio, so joins have no
 * AAC priming gaps). Existing chunks are kept: to redo a scene after a fix, delete its chunk
 * (or pass --only) and run again. The joined result is <outDir>/episode.mp4.
 */
import {execFileSync} from 'node:child_process';
import {existsSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const video = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
const planPath = args.shift(), outDir = args.shift();
if (!planPath || !outDir) throw new Error('Usage: render-episode-chunks.mjs <plan.json> <outDir> [--browser-executable p] [--only A,B] [--until-ms N]');
let browser, only, untilMs = Infinity;
while (args.length) {
  const a = args.shift();
  if (a === '--browser-executable') browser = args.shift();
  else if (a === '--only') only = new Set(args.shift().split(','));
  else if (a === '--until-ms') untilMs = Number(args.shift());
  else throw new Error(`Unknown argument: ${a}`);
}

const plan = JSON.parse(readFileSync(planPath, 'utf8'));
const fps = plan.fps ?? 30;
const frame = (ms) => Math.round(ms*fps/1000);
const segments = plan.segments.filter((s) => s.start_ms < untilMs);
const chunks = path.join(outDir, 'chunks');
mkdirSync(chunks, {recursive: true});

const files = segments.map((s, i) => {
  const file = path.join(chunks, `${String(i).padStart(2, '0')}-${s.scene}.mkv`);
  if (only?.has(s.scene)) rmSync(file, {force: true});
  if (existsSync(file)) { console.log(`keep   ${path.basename(file)}`); return file; }
  const first = frame(s.start_ms), last = frame(s.end_ms) - 1;
  console.log(`render ${path.basename(file)} frames ${first}-${last}`);
  execFileSync('npx', ['--no-install', 'remotion', 'render', 'src/stage/index.ts', 'StageEpisode', file,
    `--props=${planPath}`, `--frames=${first}-${last}`, '--codec=h264-mkv', '--audio-codec=pcm-16', '--crf=18', '--timeout=120000', '--concurrency=4',
    ...(browser ? [`--browser-executable=${browser}`] : [])], {cwd: video, stdio: 'inherit'});
  return file;
});

const list = path.join(outDir, 'chunks.txt');
writeFileSync(list, files.map((f) => `file '${f.replace(/'/g, "'\\''")}'`).join('\n') + '\n');
const out = path.join(outDir, 'episode.mp4');
execFileSync('ffmpeg', ['-loglevel', 'error', '-y', '-f', 'concat', '-safe', '0', '-i', list,
  '-c:v', 'copy', '-c:a', 'aac', '-b:a', '320k', '-movflags', '+faststart', out], {stdio: 'inherit'});
console.log(`joined ${files.length} chunks -> ${out}`);
