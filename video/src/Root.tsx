import {Episode, episodeMetadata, type EpisodePlan} from './Episode';
import samplePlan from '../examples/plan.sample.json';
import {Composition} from 'remotion';
import {TimelineCard} from './cards/TimelineCard';
import {StatBarsCard, type StatBarsProps} from './cards/StatBarsCard';
import {TitleCard} from './cards/TitleCard';
import {CompareCard} from './cards/CompareCard';
import {FreezeCallout} from './cards/FreezeCallout';
import {PartsCard} from './cards/PartsCard';
import {WIDTH, HEIGHT} from './safeZone';
import g10 from '../examples/g10.json';
import g11 from '../examples/g11.json';
const settings = {width: WIDTH, height: HEIGHT, fps: 30, durationInFrames: 1};
const episodePlan: EpisodePlan = {...samplePlan, cards: samplePlan.cards.map(c => ({...c, card: {...c.card, component: 'TitleCard' as const}}))};
export const Root = () => <>
  <Composition {...settings} id="Episode" component={Episode} defaultProps={episodePlan} calculateMetadata={episodeMetadata} />
  <Composition {...settings} id="TimelineCard" component={TimelineCard} defaultProps={{headline: 'Timeline', sources: 'Sources: Example', items: [{date: '2024', title: 'Event', detail: 'Description'}]}} />
  <Composition {...settings} id="StatBarsCard" component={StatBarsCard} defaultProps={{headline: 'Stats', sources: 'Sources: Example', items: [{label: 'A', period: 'Now', values: [10], color: 'mute'}]}} />
  <Composition {...settings} id="TitleCard" component={TitleCard} defaultProps={{headline: 'Title', sources: 'Sources: Example', items: [{title: 'Point'}]}} />
  <Composition {...settings} id="CompareCard" component={CompareCard} defaultProps={{headline: 'Compare', sources: 'Sources: Example', items: [{title: 'Left', points: ['1']}, {title: 'Right', points: ['2']}]}} />
  <Composition {...settings} id="FreezeCallout" component={FreezeCallout} defaultProps={{headline: 'Frame', sources: 'Sources: Example', image: 'assets/example.svg', aspect: 1, callouts: []}} />
  <Composition {...settings} id="PartsCard" component={PartsCard} defaultProps={{headline: 'Parts', sources: 'Sources: Example', parts: [{image: 'assets/example.svg', title: 'Part', detail: 'What it does'}]}} />
</>;
