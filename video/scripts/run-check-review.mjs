import {build} from 'esbuild';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import path from 'node:path';
const video = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const outfile = path.join(video, 'scripts/.stage-check/check-review.cjs');
await build({
  entryPoints: [path.join(video, 'scripts/check-review-determinism.tsx')],
  outfile,
  bundle: true,
  platform: 'node',
  format: 'cjs',
  jsx: 'automatic',
  alias: {
    '@stage': path.join(video, 'src/stage/api.ts'),
    '@episode': path.join(video, 'episodes/_demo/index.ts'),
  },
});
const result = spawnSync(process.execPath, [outfile, ...process.argv.slice(2)], {stdio: 'inherit'});
process.exitCode = result.status ?? 1;
