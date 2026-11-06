import katex from 'katex';

// KaTeX emits deterministic HTML during React rendering. Fonts and CSS are bundled locally.
const html = new Map<string, string>();
export function MathText({latex, display = false}: {latex: string; display?: boolean}) {
  const key = `${display}:${latex}`;
  let markup = html.get(key);
  if (!markup) {
    markup = katex.renderToString(latex, {displayMode: display, throwOnError: true, trust: false, output: 'htmlAndMathml'});
    html.set(key, markup);
  }
  return <span aria-label={latex} dangerouslySetInnerHTML={{__html: markup}} />;
}
export function InlineMath({text}: {text: string}) {
  return <>{text.split(/(\$[^$]+\$)/g).map((part, i) => part.startsWith('$') && part.endsWith('$') ? <MathText key={i} latex={part.slice(1, -1)} /> : part)}</>;
}
