import {activeTerm} from '../src/cards/EquationCard';
import {LayoutContext} from '../src/layout';
import {LANDSCAPE, layoutFromDimensions} from '../src/safeZone';
import {CardView, CaptionView, type Card} from '../src/Episode';
import {readFileSync, readdirSync} from 'node:fs';
import {join} from 'node:path';
import {isValidElement, type ReactNode} from 'react';
import {Img, staticFile} from 'remotion';
import {Frame} from '../src/cards/Frame';
import {renderToStaticMarkup} from 'react-dom/server';
import {TimelineCard} from '../src/cards/TimelineCard';
import {StatBarsCard, barWidths, type StatBarsProps} from '../src/cards/StatBarsCard';
import {TitleCard, type TitleProps} from '../src/cards/TitleCard';
import {CompareCard, type CompareProps} from '../src/cards/CompareCard';
import {WIDTH, HEIGHT, TOP, BOTTOM, SIDES, BOTTOM_RIGHT, USABLE_BOX as box} from '../src/safeZone';
import g10 from '../examples/g10.json';
import g11 from '../examples/g11.json';
import heroArt from '../examples/hero_art.json';
import titleEmphasis from '../examples/title_emphasis.json';
import {Root} from '../src/Root';
import React from 'react';

function assert(ok: boolean, message: string): asserts ok {if (!ok) throw new Error(message);}

const timeline = renderToStaticMarkup(<TimelineCard {...g10} />);
const stats = renderToStaticMarkup(<StatBarsCard {...g11 as StatBarsProps} />);
const escape = (s: string) => s.replaceAll('&', '&amp;').replaceAll("'", '&#x27;').replaceAll('"', '&quot;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
for (const s of ["The timeline they don't put in the press release", 'Chapter 135 signed', '69 DAYS LATER', 'Emergency preamble signed', '93,229 signatures submitted', '78,707 signatures verified', 'Sources: Ballotpedia · Foley Hoag LLP · AP / NBC Boston (Oct 2, 2024)']) assert(timeline.includes(escape(s)), 'Missing G-10 copy: ' + s);
for (const s of ['Louisiana, 1898: a neutral-sounding cutoff', '130,344', '5,320', '(−96%)', '164,088', '125,437', '(−24%)', 'racially neutral on its face', 'Guinn v. United States (1915)']) assert(stats.includes(escape(s)), 'Missing G-11 copy: ' + s);
// Check every JSON prose field. Dates are preserved with newlines.
function checkStrings(value: unknown, markup: string, key = '') {
  if (typeof value === 'string' && !['component', 'color', 'src', 'svgAsset'].includes(key)) assert(markup.includes(escape(value)), 'Missing example text: ' + value);
  else if (Array.isArray(value)) value.forEach(v => checkStrings(v, markup));
  else if (value && typeof value === 'object') Object.entries(value).forEach(([k,v]) => checkStrings(v, markup, k));
}
checkStrings(g10, timeline); checkStrings(g11, stats);

const heroArtMarkup = renderToStaticMarkup(<CompareCard {...heroArt as unknown as CompareProps} />);
checkStrings(heroArt, heroArtMarkup);

const titleEmphasisMarkup = renderToStaticMarkup(<TitleCard {...titleEmphasis as unknown as TitleProps} />);
checkStrings(titleEmphasis, titleEmphasisMarkup);

assert(box.y >= HEIGHT * TOP && box.y + box.height <= HEIGHT * (1 - BOTTOM), 'Vertical exclusion');
assert(box.x >= WIDTH * SIDES && box.x + box.width <= WIDTH * (1 - SIDES), 'Side exclusion');
assert(box.y + box.height <= HEIGHT * (1 - BOTTOM_RIGHT.bottom), 'Deeper overlay exclusion');
const neutral = {headline: 'Shapes', sources: 'Sources: Example dataset'};
for (const markup of [timeline, stats, renderToStaticMarkup(<TitleCard {...neutral} items={[{title: 'Circle'}]} />), renderToStaticMarkup(<CompareCard {...neutral} items={[{title: 'Square', points: ['Four sides']}]} />)]) {
  assert(markup.includes(`top:${box.y}px`) && markup.includes(`height:${box.height}px`), 'Shared safe layout missing');
  assert((markup.match(/position:absolute/g) || []).length === 1, 'Content must stay in shared bounded flow');
  assert(markup.includes('overflow:hidden'), 'Bounded content required');
  assert(markup.indexOf('Sources:') < markup.indexOf('>QR<'), 'QR must follow sources');
  assert(markup.includes('width:120px;height:120px'), 'QR size');
}

// Leakage check
const minimalStats = renderToStaticMarkup(<StatBarsCard headline="Test" sources="Sources: Test" items={[{label: 'L', period: 'P', values: [1], color: 'mute'}]} />);
assert(!minimalStats.includes('Louisiana'), 'Leakage: found Louisiana in minimal card');
assert(!minimalStats.includes('racially neutral'), 'Leakage: found quote in minimal card');

assert(JSON.stringify(barWidths([10, 5, 0])) === '[100,50,0]', 'Group maximum calculation');
assert(JSON.stringify(barWidths([0, 0])) === '[0,0]', 'Zero bars');
console.log('PASS: all G-10/G-11 copy and numbers; four card layouts within safe-zone constants; QR below sources; group-relative bar widths.');

// Each component forwards the slot into the same clipped safe-zone main container.
for (const markup of [
  renderToStaticMarkup(<TimelineCard {...g10} svgAsset="assets/example.svg" />),
  renderToStaticMarkup(<StatBarsCard {...g11 as StatBarsProps} svgAsset="assets/example.svg" />),
  renderToStaticMarkup(<TitleCard {...neutral} items={[]} svgAsset="assets/example.svg" />),
  renderToStaticMarkup(<CompareCard {...neutral} items={[]} svgAsset="assets/example.svg" />),
]) {
  const main = markup.slice(markup.indexOf('<main'), markup.indexOf('</main>'));
  assert(main.includes(`left:${box.x}px`) && main.includes(`width:${box.width}px`) && main.includes(`top:${box.y}px`) && main.includes(`height:${box.height}px`), 'Art safe-zone bounds');
  assert(main.includes('data-svg-slot') && main.includes('<img '), 'SVG slot must render an image within main');
  assert((markup.match(/position:absolute/g) || []).length === 1, 'Art must remain in bounded flow');
}

// Hero art check
const heroMarkup2 = renderToStaticMarkup(<CompareCard {...neutral} items={[]} art={{src: 'assets/example.svg', size: 'hero', caption: 'A caption'}} />);
assert(heroMarkup2.includes('height:34%'), 'Hero art height missing');
assert(heroMarkup2.includes('A caption'), 'Hero art caption missing');

// Title emphasis check
const titleMarkup2 = renderToStaticMarkup(<TitleCard {...neutral} items={[{title: 'Test', emphasis: '$100'}]} />);
assert(titleMarkup2.includes('$100'), 'Title emphasis missing');

console.log('PASS: public SVG slot renders in all four cards inside the shared safe-zone box.');

// Landscape shares copy, with a separate safe area and composition metadata.
assert(layoutFromDimensions(1920, 1080) === 'landscape', 'Landscape inference');
assert(layoutFromDimensions(1080, 1920) === 'portrait', 'Portrait inference');
assert(LANDSCAPE.box.x === 1920 * 0.05 && LANDSCAPE.box.y === 1080 * 0.06, 'Landscape margins');
assert(Math.abs(LANDSCAPE.box.y + LANDSCAPE.box.height - LANDSCAPE.captions.top) < 1e-6, 'Caption band boundary');
const landscape = (card: Card) => renderToStaticMarkup(<LayoutContext.Provider value="landscape"><CardView card={card} durationInFrames={90} /></LayoutContext.Provider>);
const compositions = React.Children.toArray(Root().props.children).filter(isValidElement) as React.ReactElement<{id: string; width: number; height: number; defaultProps: Record<string, unknown>}>[];
for (const name of ['TimelineCard', 'StatBarsCard', 'TitleCard', 'CompareCard', 'FreezeCallout', 'PartsCard', 'EquationCard', 'QuoteCard', 'TableCard']) {
  const composition = compositions.find(c => c.props.id === `${name}-16x9`);
  assert(Boolean(composition), `Missing landscape composition: ${name}`);
  assert(composition!.props.width === 1920 && composition!.props.height === 1080, `Landscape dimensions: ${name}`);
  const markup = landscape({...composition!.props.defaultProps, component: name} as Card);
  assert(markup.includes('data-layout="landscape"'), `Landscape frame: ${name}`);
}
const fixtures = process.env.YT01_CARD_FIXTURES;
if (fixtures) {
  const files = readdirSync(fixtures).filter(name => name.endsWith('.json'));
  for (const file of files) {
    const card = JSON.parse(readFileSync(join(fixtures, file), 'utf8')) as Card;
    const markup = landscape(card);
    assert(markup.includes('data-layout="landscape"'), `Fixture layout: ${file}`);
    for (const key of ['headline', 'sources', 'caption'] as const) {
      const value = (card as unknown as Record<string, unknown>)[key];
      if (typeof value === 'string') assert(markup.replace(/<[^>]*>/g, '').includes(escape(value)), `Fixture copy: ${file}/${key}`);
    }
  }
  console.log(`PASS: ${files.length} Q9 fixture cards render landscape static markup; pixel overflow requires browser rendering.`);
}
for (const [rows, columns] of [[4, 3], [6, 6]]) {
  const markup = landscape({component: 'TableCard', headline: 'Table layout', columns: Array.from({length: columns}, (_, i) => `Column ${i}`), rows: Array.from({length: rows}, () => Array.from({length: columns}, () => '$x_i$ = term')), latexCells: true, sources: 'Synthetic layout'});
  assert((markup.match(/<td /g) ?? []).length === rows * columns, 'All table cells');
  assert(markup.includes('katex-html'), 'Inline math rendered locally');
}
const equation = {component: 'EquationCard' as const, headline: 'Equation', latex: 'x+y=z', number: '2.7', terms: [{latex: 'x', gloss: 'First term'}, {latex: 'y', gloss: 'Second term'}], reading: 'The terms sum to z.', sources: 'Synthetic layout'};
const equationMarkup = landscape(equation);
assert(equationMarkup.includes('Equation 2.7') && equationMarkup.includes('katex-html'), 'Equation HTML and number');
assert((equationMarkup.match(/data-term-active="true"/g) ?? []).length === 1, 'One active equation term');
const captions = renderToStaticMarkup(<LayoutContext.Provider value="landscape"><CaptionView text="Caption" /></LayoutContext.Provider>);
assert(captions.includes('top:928.8px') && captions.includes('height:151.2px'), 'Landscape captions');
console.log('PASS: nine landscape compositions, safe margins, captions, equation HTML, and 4×3 / 6×6 table markup.');

assert([0, 29, 30, 59, 60, 89].map(frame => activeTerm(frame, 90, 3)).join(',') === '0,0,1,1,2,2', 'Equation term timing across card duration');
