import React from 'react';
import {MathText} from '../../MathText';
import {color} from '../theme/tokens';
import {DrawRect} from '../motion/DrawRect';
import type {Timing} from '../motion/core';

export type PredictiveTermHighlight = 'none' | 'evidence' | 'prior' | 'posterior' | 'equation';
export type PredictiveUpdatePlotProps = Timing & {
  evidenceMean: number;
  priorMean: number;
  precision: number;
  showEquation: boolean;
  termHighlight: PredictiveTermHighlight;
  posteriorProgress: number;
};

export const PREDICTIVE_PLOT = {width:1040,height:470,left:72,right:36,top:86,bottom:72,evidencePrecision:18} as const;
const clamp=(value:number)=>Math.max(0,Math.min(1,value));

export function priorPrecisionFor(normalizedPrecision:number) {
  const alpha=clamp(normalizedPrecision);
  return PREDICTIVE_PLOT.evidencePrecision*(.001+alpha/(1.05-alpha));
}

export function calculatePredictivePosterior(evidenceMean:number, priorMean:number, normalizedPrecision:number) {
  const evidencePrecision=PREDICTIVE_PLOT.evidencePrecision;
  const priorPrecision=priorPrecisionFor(normalizedPrecision);
  const precision=evidencePrecision+priorPrecision;
  const mean=(evidenceMean*evidencePrecision+priorMean*priorPrecision)/precision;
  return {mean, precision, evidencePrecision, priorPrecision};
}

const gaussian=(x:number,mean:number,precision:number)=>Math.exp(-.5*precision*(x-mean)*(x-mean));
const xFor=(value:number)=>PREDICTIVE_PLOT.left+clamp(value)*(PREDICTIVE_PLOT.width-PREDICTIVE_PLOT.left-PREDICTIVE_PLOT.right);
const pathFor=(mean:number,precision:number)=>Array.from({length:121},(_,index)=>{
  const x=index/120, px=xFor(x), py=PREDICTIVE_PLOT.height-PREDICTIVE_PLOT.bottom-gaussian(x,mean,precision)*230;
  return `${index?'L':'M'}${px.toFixed(2)} ${py.toFixed(2)}`;
}).join(' ');

export function PredictiveUpdatePlot({evidenceMean:rawEvidenceMean, priorMean:rawPriorMean, precision:rawPrecision, showEquation, termHighlight, posteriorProgress:rawPosteriorProgress, progress, startFrame, durationFrames}: PredictiveUpdatePlotProps) {
  const evidenceMean=clamp(rawEvidenceMean),priorMean=clamp(rawPriorMean),alpha=clamp(rawPrecision),posteriorProgress=clamp(rawPosteriorProgress);
  const posterior=calculatePredictivePosterior(evidenceMean,priorMean,alpha);
  const visiblePosteriorMean=evidenceMean+(posterior.mean-evidenceMean)*posteriorProgress;
  const axisY=PREDICTIVE_PLOT.height-PREDICTIVE_PLOT.bottom;
  const highlighted=(term:PredictiveTermHighlight)=>termHighlight===term||termHighlight==='equation';
  return <div data-shared="PredictiveUpdatePlot" data-precision={alpha.toFixed(3)} data-posterior-mode={posterior.mean.toFixed(6)} data-posterior-precision={posterior.precision.toFixed(6)} style={{position:'relative', width:PREDICTIVE_PLOT.width, height:PREDICTIVE_PLOT.height, overflow:'hidden'}}>
    <DrawRect box={{x:0,y:0,w:PREDICTIVE_PLOT.width,h:PREDICTIVE_PLOT.height}} progress={progress} startFrame={startFrame} durationFrames={durationFrames} corners="tr-bl" strokeToken="color/line/hairline" fillToken="color/fill/panel-render" glowToken="color/text/accent" seed={73}/>
    {showEquation && <div data-predictive-equation="true" style={{position:'absolute',left:72,top:24,right:36,textAlign:'center',fontSize:28,lineHeight:1.2,color:highlighted('equation')?color('color/fill/gold'):color('color/text/soft')}}><MathText text="μ_{post} = (μ_eτ_e + μ_pτ_p) / (τ_e + τ_p)"/></div>}
    <svg width={PREDICTIVE_PLOT.width} height={PREDICTIVE_PLOT.height} viewBox={`0 0 ${PREDICTIVE_PLOT.width} ${PREDICTIVE_PLOT.height}`} style={{position:'absolute',inset:0}} aria-label="Evidence, prior, and posterior distributions">
      <line x1={PREDICTIVE_PLOT.left} y1={axisY} x2={PREDICTIVE_PLOT.width-PREDICTIVE_PLOT.right} y2={axisY} stroke={color('color/line/hairline')} strokeWidth="2"/>
      <path data-curve="evidence" d={pathFor(evidenceMean,PREDICTIVE_PLOT.evidencePrecision)} fill="none" stroke={color('color/fill/gold')} strokeWidth={highlighted('evidence')?5:3} opacity={highlighted('evidence')?1:.78}/>
      <path data-curve="prior" d={pathFor(priorMean,posterior.priorPrecision)} fill="none" stroke={color('color/marker/race')} strokeWidth={highlighted('prior')?5:3} opacity={highlighted('prior')?1:.74}/>
      <path data-curve="posterior" d={pathFor(posterior.mean,posterior.precision)} fill="none" stroke={color('color/text/accent')} strokeWidth={highlighted('posterior')?6:4} opacity={posteriorProgress}/>
      <line data-posterior-marker="true" x1={xFor(visiblePosteriorMean)} y1={axisY-256*posteriorProgress} x2={xFor(visiblePosteriorMean)} y2={axisY+10} stroke={color('color/text/accent')} strokeWidth="2" opacity={posteriorProgress}/>
    </svg>
    <div style={{position:'absolute',left:xFor(evidenceMean)-82,top:axisY+18,width:164,textAlign:'center',font:'500 20px "IBM Plex Mono"',color:color('color/fill/gold')}}>EVIDENCE</div>
    <div style={{position:'absolute',left:xFor(priorMean)-72,top:axisY+18,width:144,textAlign:'center',font:'500 20px "IBM Plex Mono"',color:color('color/marker/race')}}>PRIOR</div>
    <div style={{position:'absolute',left:xFor(visiblePosteriorMean)-86,top:axisY-286,width:172,textAlign:'center',font:'500 20px "IBM Plex Mono"',color:color('color/text/accent'),opacity:posteriorProgress}}>POSTERIOR</div>
  </div>;
}
