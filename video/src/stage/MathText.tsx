import React from 'react';

export type MathRun = {kind: 'text' | 'sub' | 'sup'; text: string};

export function parseMathText(value: string): MathRun[] {
  const runs: MathRun[] = [];
  const append = (kind: MathRun['kind'], text: string) => {
    if (!text) return;
    const previous = runs[runs.length-1];
    if (previous?.kind === kind) previous.text += text;
    else runs.push({kind, text});
  };
  for (let index = 0; index < value.length;) {
    const marker = value[index];
    if (marker !== '_' && marker !== '^') {
      const point = String.fromCodePoint(value.codePointAt(index)!);
      append('text', point);
      index += point.length;
      continue;
    }
    const kind = marker === '_' ? 'sub' : 'sup';
    if (value[index+1] === '{') {
      const end = value.indexOf('}', index+2);
      if (end >= 0) {
        append(kind, value.slice(index+2, end));
        index = end+1;
        continue;
      }
    } else if (index+1 < value.length) {
      const point = String.fromCodePoint(value.codePointAt(index+1)!);
      append(kind, point);
      index += 1+point.length;
      continue;
    }
    append('text', marker);
    index++;
  }
  return runs;
}

export type MathTextProps = {children?: string; text?: string; svg?: boolean; className?: string; style?: React.CSSProperties};
export function MathText({children, text, svg = false, className, style}: MathTextProps) {
  const runs = parseMathText(text ?? children ?? '');
  if (svg) return <>{runs.map((run, index) => <tspan key={index} baselineShift={run.kind === 'sub' ? 'sub' : run.kind === 'sup' ? 'super' : 'baseline'} fontSize={run.kind === 'text' ? undefined : '0.7em'}>{run.text}</tspan>)}</>;
  return <span className={className} style={style}>{runs.map((run, index) => run.kind === 'sub'
    ? <sub key={index} style={{fontSize: '0.7em', lineHeight: 0}}>{run.text}</sub>
    : run.kind === 'sup' ? <sup key={index} style={{fontSize: '0.7em', lineHeight: 0}}>{run.text}</sup>
    : <React.Fragment key={index}>{run.text}</React.Fragment>)}</span>;
}
