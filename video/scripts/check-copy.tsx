import {renderToStaticMarkup} from 'react-dom/server';
import {TimelineCard} from '../src/cards/TimelineCard';
import {StatBarsCard, barWidths, type StatBarsProps} from '../src/cards/StatBarsCard';
import {TitleCard} from '../src/cards/TitleCard';
import {CompareCard} from '../src/cards/CompareCard';
import {WIDTH, HEIGHT, TOP, BOTTOM, SIDES, BOTTOM_RIGHT, USABLE_BOX as box} from '../src/safeZone';
import g10 from '../examples/g10.json';
import g11 from '../examples/g11.json';
function assert(ok: boolean, message: string): asserts ok {if (!ok) throw new Error(message);}
const timeline = renderToStaticMarkup(<TimelineCard {...g10} />);
const stats = renderToStaticMarkup(<StatBarsCard {...g11 as StatBarsProps} />);
const escape = (s: string) => s.replaceAll('&', '&amp;').replaceAll("'", '&#x27;').replaceAll('"', '&quot;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
for (const s of ["The timeline they don't put in the press release", 'Chapter 135 signed', '69 DAYS LATER', 'Emergency preamble signed', '93,229 signatures submitted', '78,707 signatures verified', 'Sources: Ballotpedia · Foley Hoag LLP · AP / NBC Boston (Oct 2, 2024)']) assert(timeline.includes(escape(s)), 'Missing G-10 copy: ' + s);
for (const s of ['Louisiana, 1898: a neutral-sounding cutoff', '130,344', '5,320', '(−96%)', '164,088', '125,437', '(−24%)', 'racially neutral on its face', 'Guinn v. United States (1915)']) assert(stats.includes(escape(s)), 'Missing G-11 copy: ' + s);
// Check all JSON prose, not just the acceptance subset. Dates are preserved with newlines.
function checkStrings(value: unknown, markup: string, key = '') {
  if (typeof value === 'string' && !['component', 'color'].includes(key)) assert(markup.includes(escape(value)), 'Missing example text: ' + value);
  else if (Array.isArray(value)) value.forEach(v => checkStrings(v, markup));
  else if (value && typeof value === 'object') Object.entries(value).forEach(([k,v]) => checkStrings(v, markup, k));
}
checkStrings(g10, timeline); checkStrings(g11, stats);
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
assert(JSON.stringify(barWidths([10, 5, 0])) === '[100,50,0]', 'Group maximum calculation');
assert(JSON.stringify(barWidths([0, 0])) === '[0,0]', 'Zero bars');
console.log('PASS: all G-10/G-11 copy and numbers; four card layouts within safe-zone constants; QR below sources; group-relative bar widths.');
