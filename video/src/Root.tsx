import {Episode, episodeMetadata, type EpisodePlan} from './Episode';
import samplePlan from '../examples/plan.sample.json';
import {Composition} from 'remotion';
import {TimelineCard} from './cards/TimelineCard';
import {StatBarsCard, type StatBarsProps} from './cards/StatBarsCard';
import {TitleCard} from './cards/TitleCard';
import {CompareCard} from './cards/CompareCard';
import {WIDTH, HEIGHT} from './safeZone';
import g10 from '../examples/g10.json';
import g11 from '../examples/g11.json';
const settings = {width: WIDTH, height: HEIGHT, fps: 30, durationInFrames: 1};
const episodePlan: EpisodePlan = {...samplePlan, cards: samplePlan.cards.map(c => ({...c, card: {...c.card, component: 'TitleCard' as const}}))};
export const Root = () => <>
  <Composition {...settings} id="Episode" component={Episode} defaultProps={episodePlan} calculateMetadata={episodeMetadata} />
  <Composition {...settings} id="TimelineCard" component={TimelineCard} defaultProps={g10} />
  <Composition {...settings} id="StatBarsCard" component={StatBarsCard} defaultProps={g11 as StatBarsProps} />
  <Composition {...settings} id="TitleCard" component={TitleCard} defaultProps={{headline: 'Small shapes', items: [{title: 'Circle', detail: 'A simple outline.'}], sources: 'Sources: Example dataset'}} />
  <Composition {...settings} id="CompareCard" component={CompareCard} defaultProps={{headline: 'Two shapes', items: [{title: 'Circle', points: ['Round']}, {title: 'Square', points: ['Four sides']}], sources: 'Sources: Example dataset'}} />
</>;
