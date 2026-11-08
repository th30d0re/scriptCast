import {build} from 'esbuild';
import {createRequire} from 'node:module';
import {resolve, dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
const video=resolve(dirname(fileURLToPath(import.meta.url)), '..');
const modules={kit:'./src/stage/kit/index',api:'./src/stage/api',motion:'./src/stage/kit/motion/index'};
for(const name of ['DrawRect','DrawLine','Flicker','FitBox','BootIn','StickFigure','Trace','KitMotion']) modules[name]=`./src/stage/kit/motion/${name}`;
for(const name of ['DrawRect','DrawLine','Flicker','FitBox','BootIn']) modules[`opus/${name}`]=`./src/stage/kit/motion/${name}.opus`;
modules.astra='./src/stage/kit/motion/variants/astra/primitives';
for(const name of ['DrawRect','DrawLine','Flicker','FitBox','BootIn']) modules[`kimi/${name}`]=`./src/stage/kit/motion/variants/kimi/${name}`;
for(const name of ['DrawRect','Flicker','FitBox']) modules[`flash/${name}`]=`./src/stage/kit/motion/variants/flash/${name}`;
const names=['DrawRect','DrawLine','Flicker','PowerDown','FitBox','BootIn','StickFigure','Trace','KitMotion'];
const source=Object.entries(modules).map(([key,path],i)=>`import * as m${i} from '${path}';`).join('\n')+`\nexport const modules={${Object.keys(modules).map((key,i)=>`${JSON.stringify(key)}:m${i}`).join(',')}};`;
const result=await build({stdin:{contents:source,resolveDir:video,loader:'ts'},bundle:true,platform:'node',format:'cjs',write:false,loader:{'.css':'empty'},packages:'external',alias:{'@scriptcast/cards':resolve(video,'../packages/cards/src/index.ts')}});
const module={exports:{}};
const require=createRequire(resolve(video,'package.json'));
const React=require('react');
const {renderToStaticMarkup}=require('react-dom/server');
const remotion=require('remotion');
let frame=0;
// Only substitute the clock: use the real React renderer and real JSX components.
new Function('require','module','exports',result.outputFiles[0].text)(id=>id.endsWith('.css')?{}:id==='remotion'?{...remotion,useCurrentFrame:()=>frame}:require(id),module,module.exports);
for(const [key,value] of Object.entries(module.exports.modules)){
  const leaf=key.split('/').at(-1);
  const expected=['kit','api','motion'].includes(key)?names
    :key==='astra'?['DrawRect','DrawLine','Flicker','FitBox','BootIn']
    :leaf==='Flicker'&&key!=='flash/Flicker'?['Flicker','PowerDown']:[leaf];
  if(!expected.length) throw new Error(`${key}: no component exports`);
  for(const name of expected){
    console.log(`${key}.${name}: ${typeof value[name]}`);
    if(typeof value[name]!=='function') throw new Error(`${key}.${name} is not a component`);
  }
}
const kit=module.exports.modules.kit;
for(frame=0;frame<300;frame++) renderToStaticMarkup(React.createElement(kit.KitMotion));
console.log('KitMotion: rendered all 300 frames (including 200 and 299) with a controlled clock.');
const common={box:{x:0,y:0,w:300,h:200},points:[[0,0],[100,50]],text:'CHECK',x:0,y:0,progress:0.75,children:React.createElement('span',null,'content')};
for(const name of ['DrawRect','DrawLine','Flicker','PowerDown','FitBox','BootIn']) {
  for(const variant of name==='FitBox'?['opus','astra','kimi','flash']:['opus','astra','kimi']) {
    for(const mode of name==='BootIn'?['snap','scan','glitch']:[undefined]) renderToStaticMarkup(React.createElement(kit[name],{...common,variant,mode}));
  }
}
console.log('All wrapper variants and BootIn modes rendered without invalid element types.');
// The 4.0.354 renderer mounts this element before it ever mounts the composition.
const studioVersion=require('@remotion/studio/package.json').version;
const runtimeVersion=require('remotion/package.json').version;
console.log(`Remotion runtime ${runtimeVersion}; Studio ${studioVersion}; Internals.RemotionRoot: ${typeof remotion.Internals.RemotionRoot}`);
if(studioVersion!==runtimeVersion) throw new Error('Remotion runtime and Studio versions must match.');
if(studioVersion==='4.0.354'&&typeof remotion.Internals.RemotionRoot!=='function') throw new Error('Studio render root is undefined.');
