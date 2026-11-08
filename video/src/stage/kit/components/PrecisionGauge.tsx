import React from 'react';
import {color} from '../theme/tokens';

export type PrecisionGaugeProps = {
  value: number;
  derived: boolean;
  label: string;
  lowLabel: string;
  highLabel: string;
};

export function PrecisionGauge({value:rawValue, derived, label, lowLabel, highLabel}: PrecisionGaugeProps) {
  const value=Math.max(0,Math.min(1,rawValue));
  return <div data-shared="PrecisionGauge" data-gauge-value={value.toFixed(3)} style={{position:'relative',width:'100%',height:'100%',minWidth:300,minHeight:250,boxSizing:'border-box',padding:'30px 30px 24px',background:color('color/fill/panel-render'),border:`1px solid ${color('color/line/hairline')}`}}>
    <div style={{font:'500 20px "IBM Plex Mono"',letterSpacing:1,color:color('color/text/soft')}}>{label}</div>
    {derived && <div style={{position:'absolute',right:28,top:30,font:'400 20px "IBM Plex Mono"',color:color('color/text/accent')}}>DERIVED</div>}
    <div style={{position:'absolute',left:34,right:34,top:106,height:18,background:color('color/surface/well'),border:`1px solid ${color('color/line/hairline')}`}}>
      <div style={{height:'100%',width:`${value*100}%`,background:color('color/fill/gold'),transformOrigin:'left center'}}/>
      <div data-gauge-marker="true" style={{position:'absolute',left:`${value*100}%`,top:-12,width:3,height:42,background:color('color/text/accent'),transform:'translateX(-1px)'}}/>
    </div>
    <div style={{position:'absolute',left:30,top:144,font:'400 20px "IBM Plex Mono"',color:color('color/text/muted')}}>{lowLabel}</div>
    <div style={{position:'absolute',right:30,top:144,font:'400 20px "IBM Plex Mono"',color:color('color/text/muted')}}>{highLabel}</div>
    <div style={{position:'absolute',left:0,right:0,bottom:24,textAlign:'center',font:'500 32px "IBM Plex Mono"',color:color('color/text/primary')}}>{value.toFixed(2)}</div>
  </div>;
}

