import React from 'react';
import {AbsoluteFill, useCurrentFrame} from 'remotion';
import {color, Tokens} from '../theme/tokens';
import {DrawRect} from './DrawRect';
import {DrawLine} from './DrawLine';
import {Flicker, PowerDown} from './Flicker';
import {FitBox} from './FitBox';
import {StickFigure} from './StickFigure';
import {BootIn} from './BootIn';
import {KitSound} from './KitSound';

const variants=['opus','astra','kimi'] as const;
const fitVariants=['opus','astra','kimi','flash'] as const;
const sectionNames=['DrawRect','DrawLine','Flicker + PowerDown','FitBox','StickFigure','BootIn'] as const;
const Cell=({label,index,children}:{label:string;index:number;children:React.ReactNode})=><div style={{position:'absolute',left:70+index*445,top:210,width:400,height:720,border:`1px solid ${color('color/line/graticule')}`,background:color('color/surface/1'),overflow:'hidden'}}><div style={{height:54,padding:'16px 20px',fontFamily:Tokens.fonts.mono,fontSize:18,letterSpacing:2,color:color('color/text/accent'),textTransform:'uppercase',boxSizing:'border-box'}}>{label}</div>{children}</div>;
const Panel=({children}:{children:React.ReactNode})=><div style={{position:'absolute',left:28,top:80,width:344,height:560}}>{children}</div>;

export function KitMotion(){
  const frame=useCurrentFrame(), section=Math.min(5,Math.floor(frame/50)), startFrame=section*50;
  const labels=section===3?fitVariants:section===4?(['opus'] as const):variants;
  return <KitSound><AbsoluteFill style={{background:color('color/bg/field'),fontFamily:Tokens.fonts.sans,color:color('color/text/primary')}}>
    <div style={{position:'absolute',left:70,top:58,fontSize:42,fontWeight:600}}>Stage kit motion</div>
    <div style={{position:'absolute',left:72,top:116,fontFamily:Tokens.fonts.mono,fontSize:18,color:color('color/text/muted'),letterSpacing:2}}>{sectionNames[section]} · DEFAULT + SELECTED ALTERNATES</div>
    {labels.map((variant,index)=><Cell key={variant} label={variant==='opus'?'OPUS · DEFAULT':variant} index={index}><Panel>
      {section===0&&<DrawRect variant={variant as 'opus'|'astra'|'kimi'} box={{x:45,y:130,w:255,h:210}} notch="top" startFrame={startFrame}/>}
      {section===1&&<DrawLine variant={variant as 'opus'|'astra'|'kimi'} points={[[35,330],[125,140],[230,245],[315,95]]} end="arrow" startFrame={startFrame}/>}
      {section===2&&<><Flicker variant={variant as 'opus'|'astra'|'kimi'} startFrame={startFrame} seed={17}><div style={{position:'absolute',left:48,top:120,width:250,height:130,display:'grid',placeItems:'center',background:color('color/fill/panel-wash'),border:`1px solid ${color('color/line/wire')}`,fontFamily:Tokens.fonts.mono}}>POWER ON</div></Flicker><PowerDown variant={variant as 'opus'|'astra'|'kimi'} startFrame={startFrame} seed={23} box={{x:48,y:330,w:250,h:130}}><div style={{position:'absolute',left:48,top:330,width:250,height:130,display:'grid',placeItems:'center',background:color('color/fill/panel-wash'),border:`1px solid ${color('color/line/wire')}`,fontFamily:Tokens.fonts.mono}}>POWER DOWN</div></PowerDown></>}
      {section===3&&<FitBox variant={variant as 'opus'|'astra'|'kimi'|'flash'} text="SYSTEM READY" x={32} y={210} family="mono" size={25} tag="SYS·01" startFrame={startFrame}/>}
      {section===4&&<><StickFigure x={100} y={260} group="ingroup" emphasis={1} scale={2.2} pose="open" startFrame={startFrame}/><StickFigure x={250} y={260} group="outgroup" emphasis={.85} scale={2.2} pose="raise" startFrame={startFrame}/></>}
      {section===5&&<BootIn variant={variant as 'opus'|'astra'|'kimi'} mode="scan" box={{x:30,y:105,w:285,h:330}} startFrame={startFrame}><div style={{width:'100%',height:'100%',display:'grid',placeItems:'center',background:color('color/fill/panel-wash'),fontFamily:Tokens.fonts.mono,fontSize:22}}>BOOT COMPLETE</div></BootIn>}
    </Panel></Cell>)}
    <div style={{position:'absolute',right:62,bottom:42,fontFamily:Tokens.fonts.mono,fontSize:15,color:color('color/text/muted')}}>FRAME {frame.toString().padStart(3,'0')} / 299</div>
  </AbsoluteFill></KitSound>;
}
