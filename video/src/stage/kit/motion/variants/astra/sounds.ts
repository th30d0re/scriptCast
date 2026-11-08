/** Reference timeline: 30 frames at 30fps. Scale offsets by durationFrames / 30.
 * Nested components inherit the parent's orchestration: do not double-trigger child cues.
 * These are operator cues, not bundled audio. Gain is relative to the earshot source.
 */
export type SoundCue={component:string;event:string;frameOffset:number;sound:string;gain_db:number;note:string};
const cue=(component:string,event:string,frameOffset:number,sound:string,gain_db:number,note:string):SoundCue=>({component,event,frameOffset,sound,gain_db,note});
export const sounds:SoundCue[]=[
 cue('DrawRect','corner start',0,'fit/tick',-27,'One tick for both opposing origins.'),
 cue('DrawRect','lines meet',21,'soft click',-29,'Wire closure; no impact boom.'),
 cue('DrawRect','fill on',30,'hover_tick/air',-34,'Wash settles after closure; intermediate fill flutter stays silent.'),
 cue('DrawLine','head launch',0,'swish/soft',-32,'Short travel bed; one per path, not per segment.'),
 cue('DrawLine','head settles',30,'hover_tick/detent',-31,'Front dims into the finished trace.'),
 ...[0,7,12,18].map((f,i)=>cue('Flicker',i===0?'power on':`pulse ${i}`,f,i===0?'fit/switch_flip':'glitch/chirp',i===0?-30:-38,'Sparse envelope edges; hash modulates brightness only.')),
 cue('Flicker','steady',30,'hover_tick/air',-36,'Optional breath at full power.'),
 ...[0,13,19,24,30].map((f,i)=>cue('Flicker.off',i===4?'power off':`decay ${i}`,f,i===4?'soft click':'glitch/dropout',-37,'Reverse envelope; keep underneath speech.')),
 cue('FitBox','corner start',0,'fit/tick',-29,'Owns nested DrawRect cues.'),
 cue('FitBox','lines meet',15,'soft click',-32,'Measured enclosure closes.'),
 cue('FitBox','fill on',21,'hover_tick/air',-36,'Background locks.'),
 ...[17,20,22,25].map((f,i)=>cue('FitBox',i===0?'text reveal':`text pulse ${i}`,f,'fit/key/lubed',-37,'One quiet clustered text power-on, never per character.')),
 cue('FitBox','steady',30,'hover_tick/air',-38,'Text holds.'),
 cue('StickFigure','head draw',0,'fit/tick',-33,'One per group, not per individual in a crowd.'),
 ...[4,6,8,10,12].map((f,i)=>cue('StickFigure',`limb ${i+1}`,f,'hover_tick/air',-40,'Optional very quiet stroke onset.')),
 cue('StickFigure','figure complete',24,'soft click',-35,'Last leg closes the silhouette.'),
 ...(['snap','scan','glitch'] as const).flatMap(variant=>[
 cue(`BootIn.${variant}`,'corner start',0,'fit/tick',-29,'Preset owns all nested cues.'),
 cue(`BootIn.${variant}`,'lines meet',variant==='snap'?7:11,'soft click',-33,'Border completes before power.'),
 cue(`BootIn.${variant}`,'fill on',variant==='snap'?9:15,'hover_tick/air',-39,'Enclosure wash finishes.'),
 ...[.23,.4,.59].map((pulse,i)=>cue(`BootIn.${variant}`,`power pulse ${i+1}`,Math.round(30*((variant==='snap'?.3:.48)+pulse*(variant==='snap'?.34:.16))),'glitch/chirp',-40,'Quiet nested flicker edge.')),
 cue(`BootIn.${variant}`,'power',variant==='snap'?9:15,variant==='glitch'?'glitch/data':'fit/switch_flip',-35,'Short power envelope; no sustained noise.'),
 cue(`BootIn.${variant}`,'content reveal',20,variant==='scan'?'swish/soft':'fit/key/lubed',-33,'Scan front or content rise.'),
 cue(`BootIn.${variant}`,'settled',30,'hover_tick/air',-38,'Hold without looping audio.')])
];
export default sounds;
/** Reel placements; scale each component's reference cues by durationFrames / 30. */
export const reelSoundInstances = [
 {component:'DrawRect',startFrame:0,durationFrames:28},
 {component:'DrawLine',startFrame:35,durationFrames:28},
 {component:'Flicker',startFrame:70,durationFrames:16},
 {component:'Flicker.off',startFrame:93,durationFrames:16},
 {component:'FitBox',startFrame:115,durationFrames:28},
 {component:'FitBox',startFrame:123,durationFrames:24},
 ...[0,1,2].map(i=>({component:'StickFigure',startFrame:150+i*3,durationFrames:22})),
 ...(['snap','scan','glitch'] as const).map((v,i)=>({component:`BootIn.${v}`,startFrame:180+i*7,durationFrames:30}))
];
export const reelSounds = reelSoundInstances.flatMap(instance=>sounds.filter(c=>c.component===instance.component).map(c=>({...c,frameOffset:instance.startFrame+Math.round(c.frameOffset*instance.durationFrames/30)})));
