// The five-tier notch in the card border.
import {cubicInOut,cubicOut,progress} from '../../motion';

// Geometry from tools/svg_to_tsx.mjs: the pre-Stage notch scaled 0.8, wire at y0, dir +1 down / -1 up.
const CX=960,KINK=62.4,APEX=10.7,DEPTH=48,BARS:[number,number][]=[[7.8,40.9],[17.4,32.8],[27,24.9],[36.6,16.9]];
const r=(n:number)=>+n.toFixed(2);

// p = 1 is the full notch; p = 0 is a straight wire. The V dips first, then the bars grow base first.
export function notchPath(y0:number,dir:number,p:number):string {
 const a=cubicInOut(progress(p,0,.55)),y=(d:number)=>r(y0+dir*d*a);
 let d=`M786.059 ${y0}H${r(CX-KINK)}L${r(CX-APEX*a)} ${y(DEPTH)}M${r(CX+APEX*a)} ${y(DEPTH)}L${r(CX+KINK)} ${y0}H1133.94`;
 BARS.forEach(([dy,h],i)=>{const b=cubicOut(progress(p,.44+.12*i,.64+.12*i));if(b>0)d+=`M${r(CX-h*b)} ${y(dy)}H${r(CX+h*b)}`;});
 return d;
}
