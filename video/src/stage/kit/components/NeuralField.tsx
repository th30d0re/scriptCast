import React from 'react';
import {color} from '../theme/tokens';
import {DrawRect} from '../motion/DrawRect';
import {clamp01, cubicOut, useTiming, type Timing} from '../motion/core';

export type NeuralFieldProps = Timing & {
  layers: number | readonly number[];
  inputs: readonly string[];
  weights: readonly number[] | readonly (readonly number[])[];
  activationProgress: number;
  compressionProgress: number;
  labels: readonly string[];
};

export const NEURAL_FIELD = {width:920,height:460,padX:92,padY:72} as const;
const clamp=(value:number)=>Math.max(0,Math.min(1,value));

function countsFor(layers:NeuralFieldProps['layers'],inputs:readonly string[],labels:readonly string[]) {
  if (Array.isArray(layers)) return [...layers];
  const count=Math.max(2,Math.round(layers as number));
  return Array.from({length:count},(_,index)=>index===0?Math.max(2,inputs.length):index===count-1?Math.max(2,labels.length):4);
}

function flatWeights(weights:NeuralFieldProps['weights']) {
  return weights.flatMap(value=>Array.isArray(value)?[...value]:[value as number]);
}

export function NeuralField({layers, inputs, weights, activationProgress:rawActivation, compressionProgress:rawCompression, labels, progress, startFrame, durationFrames}: NeuralFieldProps) {
  // Untimed means fully shown (kit convention).
  if (progress === undefined && startFrame === undefined) progress = 1;
  const duration=durationFrames??42, {p:timingProgress}=useTiming({progress,startFrame,durationFrames:duration},duration);
  const activationSweep=cubicOut(clamp01((timingProgress-.52)/.48));
  const activation=clamp(rawActivation)*activationSweep,compression=clamp(rawCompression)*timingProgress,counts=countsFor(layers,inputs,labels),allWeights=flatWeights(weights);
  const edgeReveal=(index:number,total:number)=>cubicOut(clamp01((timingProgress*.78-index/Math.max(1,total)*.48)/.30));
  const nodeReveal=(layer:number,index:number)=>cubicOut(clamp01((timingProgress-.34-layer*.09-index*.025)/.28));
  const labelReveal=cubicOut(clamp01((timingProgress-.72)/.28));
  const positions=counts.map((count,layer)=>Array.from({length:count},(_,index)=>({
    x:NEURAL_FIELD.padX+layer*(NEURAL_FIELD.width-2*NEURAL_FIELD.padX)/Math.max(1,counts.length-1),
    y:NEURAL_FIELD.padY+index*(NEURAL_FIELD.height-2*NEURAL_FIELD.padY)/Math.max(1,count-1),
  })));
  const totalWires=positions.slice(0,-1).reduce((total,layer,layerIndex)=>total+layer.length*positions[layerIndex+1].length,0);
  // One transform places a node and every wire end that meets it, so compression never separates them.
  const shifted=(point:{x:number;y:number},layerIndex:number)=>({
    x:point.x-40*compression*(layerIndex/Math.max(1,counts.length-1)),
    y:point.y+(NEURAL_FIELD.height/2-point.y)*.22*compression,
  });
  let wireIndex=0;
  return <div data-shared="NeuralField" style={{position:'relative', width:NEURAL_FIELD.width, height:NEURAL_FIELD.height, overflow:'hidden'}}>
    <DrawRect box={{x:0,y:0,w:NEURAL_FIELD.width,h:NEURAL_FIELD.height}} progress={progress} startFrame={startFrame} durationFrames={duration} corners="tr-bl" strokeToken="color/line/hairline" fillToken="color/fill/panel-render" glowToken="color/fill/gold" seed={31}/>
    <svg width={NEURAL_FIELD.width} height={NEURAL_FIELD.height} viewBox={`0 0 ${NEURAL_FIELD.width} ${NEURAL_FIELD.height}`} style={{position:'absolute',inset:0}} aria-label="Neural processing field">
      {positions.slice(0,-1).flatMap((layer,layerIndex)=>layer.flatMap(start=>positions[layerIndex+1].map(end=>{
        const weight=allWeights.length?allWeights[wireIndex%allWeights.length]:.5, index=wireIndex++;
        const strength=.12+.68*clamp(Math.abs(weight))*activation;
        const a=shifted(start,layerIndex), b=shifted(end,layerIndex+1);
        const reveal=edgeReveal(index, totalWires);
        const length=Math.hypot(b.x-a.x,b.y-a.y);
        return <line key={`w-${index}`} data-neural-wire={index} x1={a.x} y1={a.y} x2={b.x} y2={b.y} stroke={weight<0?color('color/marker/race'):color('color/line/wire')} strokeWidth={1+3*strength} opacity={strength*reveal} strokeDasharray={length} strokeDashoffset={length*(1-reveal)}/>;
      })))}
      {positions.flatMap((layer,layerIndex)=>layer.map((point,index)=>{
        const pulse=clamp(activation*counts.length-layerIndex+.15*index);
        const {x,y}=shifted(point,layerIndex);
        const reveal=nodeReveal(layerIndex,index);
        return <g key={`n-${layerIndex}-${index}`} data-neural-node={`${layerIndex}-${index}`} opacity={reveal} transform={`translate(${x} ${y}) scale(${.45+.55*reveal}) translate(${-x} ${-y})`}>
          <circle cx={x} cy={y} r={9+5*pulse} fill={color('color/bg/field')} stroke={pulse>.5?color('color/fill/gold'):color('color/line/hairline')} strokeWidth={2}/>
          <circle cx={x} cy={y} r={4} fill={color('color/fill/gold')} opacity={pulse}/>
        </g>;
      }))}
    </svg>
    {inputs.map((input,index)=><div key={input} style={{position:'absolute',left:10,top:42+index*32,font:'400 20px "IBM Plex Mono"',color:color('color/text/muted'),opacity:labelReveal}}>{input}</div>)}
    {labels.map((label,index)=><div key={label} style={{position:'absolute',right:12,bottom:18+index*28,font:'400 20px "IBM Plex Mono"',color:color('color/text/soft'),opacity:labelReveal}}>{label}</div>)}
  </div>;
}
