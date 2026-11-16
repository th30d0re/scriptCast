#!/usr/bin/env node
import {execFileSync} from 'node:child_process';
import {mkdirSync, readFileSync, readdirSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const motionDir=join(root,'src/stage/kit/motion');
const outDir=join(root,'public/sfx');
const earshot=process.env.EARSHOT_BIN ?? '/Users/emmanuel/Dev/Tools/earshot/.venv/bin/earshot';
const force=process.argv.includes('--force');
const env={...process.env,NUMBA_CACHE_DIR:process.env.NUMBA_CACHE_DIR ?? '/tmp/earshot-numba',EARSHOT_HOME:process.env.EARSHOT_HOME ?? '/tmp/earshot-cache'};
const run=(args,options={})=>execFileSync(earshot,args,{encoding:'utf8',stdio:['ignore','pipe','pipe'],env,...options});
const check=(output)=>{
  try { return run(['sfx','check',output]); }
  catch (error) { return `${error.stdout?.toString() ?? ''}${error.stderr?.toString() ?? ''}`; }
};

function soundTables(dir) {
  return readdirSync(dir,{withFileTypes:true}).flatMap((entry)=>entry.isDirectory()
    ? soundTables(join(dir,entry.name))
    : /^sounds(?:\.[^.]+)?\.ts$/.test(entry.name) ? [join(dir,entry.name)] : []);
}

const bank=new Set(run(['sfx','list']).split('\n').slice(2).map((line)=>line.trim().split(/\s+/)[0]).filter(Boolean));
// The Opus file also declares the complete bank-id union. Remove that type catalog so
// only ids actually referenced by cue definitions are selected.
const source=soundTables(motionDir).map((file)=>readFileSync(file,'utf8')).join('\n').replace(/export type EarshotId\s*=.*?;\n/s,'');
const ids=[...bank].filter((id)=>source.includes(`'${id}'`) || source.includes(`"${id}"`)).sort();
if (!ids.length) throw new Error('No Earshot sound ids found in motion sound tables.');
mkdirSync(outDir,{recursive:true});

let failures=0;
for (const id of ids) {
  const output=join(outDir,`${id.replaceAll('/','__')}.wav`);
  try {
    let exists=true;
    try { readFileSync(output); } catch { exists=false; }
    if (force || !exists) run(['sfx','render',id,'-o',output,'--sr','48000'],{stdio:'inherit'});
    let checked='';
    let duration=2;
    for (;;) {
      checked=check(output);
      if (!/cut[ -]?tail|tail[^\n]*(?:cut|truncat)|truncat[^\n]*tail/i.test(checked)) break;
      if (duration > 8) throw new Error(`cut-tail warning remains after 8s render\n${checked}`);
      run(['sfx','render',id,'-o',output,'--sr','48000','--dur',String(duration)],{stdio:'inherit'});
      duration*=2;
    }
    process.stdout.write(`${/clean/i.test(checked) ? 'ok' : 'warning'} ${id}\n${checked}`);
  } catch (error) {
    failures++;
    console.error(`FAILED ${id}: ${error.stderr?.toString() ?? error.message}`);
  }
}
console.log(`${ids.length-failures}/${ids.length} sounds rendered and checked.`);
if (failures) process.exitCode=1;
