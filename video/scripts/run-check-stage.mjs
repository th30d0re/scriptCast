import {build} from 'esbuild';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import path from 'node:path';
const video=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const outfile=path.join(video,'scripts/.stage-check/check.cjs');
await build({entryPoints:[path.join(video,'scripts/check-stage.ts')],outfile,bundle:true,platform:'node',format:'cjs'});
const result=spawnSync(process.execPath,[outfile],{stdio:'inherit'});
process.exitCode=result.status??1;
