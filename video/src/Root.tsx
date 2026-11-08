import {Episode, episodeMetadata, type EpisodePlan} from './Episode';
import samplePlan from '../examples/plan.sample.json';
import {Composition} from 'remotion';
import {TimelineCard, StatBarsCard, TitleCard, CompareCard, FreezeCallout, PartsCard, EquationCard, QuoteCard, TableCard} from './cards';
import {WIDTH, HEIGHT} from '@scriptcast/cards';
import g10 from '../examples/g10.json';
import g11 from '../examples/g11.json';
import {TimelineComposition} from './timeline/TimelineComposition';
import {KitGallery, KitGallery2, KitMotion} from './stage/kit';
const settings = {width: WIDTH, height: HEIGHT, fps: 30, durationInFrames: 1};
const episodePlan: EpisodePlan = {...samplePlan, cards: samplePlan.cards.map(c => ({...c, card: {...c.card, component: 'TitleCard' as const}}))};
const equation = {headline: 'Equation walkthrough', latex: String.raw`U = I \sqcup O`, number: '2.1', terms: [{latex: 'U', gloss: 'Universe'}, {latex: 'I', gloss: 'In-group'}, {latex: 'O', gloss: 'Out-group'}], reading: 'The universe is partitioned into the in-group and out-group.', sources: 'Example layout'};
const quote = {headline: 'Quotation', quote: 'Example quotation for layout preview.', speaker: 'Example speaker', work: 'Example work', year: '2026', sources: 'Example layout'};
const table = {headline: 'Six-column table', columns: ['Scale', 'Elite', 'Puppet', 'Enforcement', 'Buffer', 'Out-group'], rows: Array.from({length: 6}, (_, i) => [`Example ${i + 1}`, 'Tier E', 'Tier P', 'Tier F', 'Tier I', 'Tier O']), sources: 'Synthetic layout example'};
export const Root = () => <>
  <Composition id="KitGallery" component={KitGallery} durationInFrames={90} fps={30} width={1920} height={1080}/>
  <Composition id="KitGallery2" component={KitGallery2} durationInFrames={90} fps={30} width={1920} height={1080}/>
  <Composition id="KitMotion" component={KitMotion} durationInFrames={300} fps={30} width={1920} height={1080}/>
  <Composition {...settings} id="Episode" component={Episode} defaultProps={episodePlan} calculateMetadata={episodeMetadata} />
  <Composition {...settings} id="TimelineCard" component={TimelineCard} defaultProps={{headline: 'Timeline', sources: 'Sources: Example', items: [{date: '2024', title: 'Event', detail: 'Description'}]}} />
  <Composition {...settings} id="StatBarsCard" component={StatBarsCard} defaultProps={{headline: 'Stats', sources: 'Sources: Example', items: [{label: 'A', period: 'Now', values: [10], color: 'mute'}]}} />
  <Composition {...settings} id="TitleCard" component={TitleCard} defaultProps={{headline: 'Title', sources: 'Sources: Example', items: [{title: 'Point'}]}} />
  <Composition {...settings} id="CompareCard" component={CompareCard} defaultProps={{headline: 'Compare', sources: 'Sources: Example', items: [{title: 'Left', points: ['1']}, {title: 'Right', points: ['2']}]}} />
  <Composition {...settings} id="FreezeCallout" component={FreezeCallout} defaultProps={{headline: 'Frame', sources: 'Sources: Example', image: 'assets/example.svg', aspect: 1, callouts: []}} />
  <Composition {...settings} id="PartsCard" component={PartsCard} defaultProps={{headline: 'Parts', sources: 'Sources: Example', parts: [{image: 'assets/example.svg', title: 'Part', detail: 'What it does'}]}} />
  <Composition {...settings} id="EquationCard" component={EquationCard} defaultProps={equation} />
  <Composition {...settings} id="QuoteCard" component={QuoteCard} defaultProps={quote} />
  <Composition {...settings} id="TableCard" component={TableCard} defaultProps={table} />
  <Composition {...settings} width={1920} height={1080} id="TimelineCard-16x9" component={TimelineCard} defaultProps={{headline: 'Timeline', sources: 'Sources: Example', items: [{date: '2024', title: 'Event', detail: 'Description'}]}} />
  <Composition {...settings} width={1920} height={1080} id="StatBarsCard-16x9" component={StatBarsCard} defaultProps={{headline: 'Stats', sources: 'Sources: Example', items: [{label: 'A', period: 'Now', values: [10], color: 'mute'}]}} />
  <Composition {...settings} width={1920} height={1080} id="TitleCard-16x9" component={TitleCard} defaultProps={{headline: 'Title', sources: 'Sources: Example', items: [{title: 'Point'}]}} />
  <Composition {...settings} width={1920} height={1080} id="CompareCard-16x9" component={CompareCard} defaultProps={{headline: 'Compare', sources: 'Sources: Example', items: [{title: 'Left', points: ['1']}, {title: 'Right', points: ['2']}]}} />
  <Composition {...settings} width={1920} height={1080} id="FreezeCallout-16x9" component={FreezeCallout} defaultProps={{headline: 'Frame', sources: 'Sources: Example', image: 'assets/example.svg', aspect: 1, callouts: []}} />
  <Composition {...settings} width={1920} height={1080} id="PartsCard-16x9" component={PartsCard} defaultProps={{headline: 'Parts', sources: 'Sources: Example', parts: [{image: 'assets/example.svg', title: 'Part', detail: 'What it does'}]}} />
  <Composition {...settings} width={1920} height={1080} id="EquationCard-16x9" component={EquationCard} defaultProps={equation} />
  <Composition {...settings} width={1920} height={1080} id="QuoteCard-16x9" component={QuoteCard} defaultProps={quote} />
  <Composition {...settings} width={1920} height={1080} id="TableCard-16x9" component={TableCard} defaultProps={table} />
  <TimelineComposition />
</>;
