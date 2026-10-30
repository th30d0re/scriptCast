import * as cards from '@scriptcast/cards';
import type {BaseProps} from '@scriptcast/cards';
import type {ComponentType} from 'react';
import example from '../../video/examples/timeline.example.json';
import {FitCard} from './FitCard';
import {useScrollProgress} from './useScrollProgress';

// Example props: the timeline example where it has the card, else the Studio defaults
// from video/src/Root.tsx.
const fromExample = (card: string) => example.items.find(item => item.card === card)?.props;
const fallback = {
  TimelineCard: {headline: 'Timeline', sources: 'Sources: Example', items: [{date: '2024', title: 'Event', detail: 'Description'}]},
  StatBarsCard: {headline: 'Stats', sources: 'Sources: Example', items: [{label: 'A', period: 'Now', values: [10], color: 'mute'}]},
  TitleCard: {headline: 'Title', sources: 'Sources: Example', items: [{title: 'Point'}]},
  CompareCard: {headline: 'Compare', sources: 'Sources: Example', items: [{title: 'Left', points: ['1']}, {title: 'Right', points: ['2']}]},
  FreezeCallout: {headline: 'Frame', sources: 'Sources: Example', image: 'assets/example.svg', aspect: 1, callouts: []},
  PartsCard: {headline: 'Parts', sources: 'Sources: Example', parts: [{image: 'assets/example.svg', title: 'Part', detail: 'What it does'}]},
};

type Section = {name: keyof typeof fallback; Card: ComponentType<any>};
const sections: Section[] = [
  {name: 'TitleCard', Card: cards.TitleCard},
  {name: 'TimelineCard', Card: cards.TimelineCard},
  {name: 'StatBarsCard', Card: cards.StatBarsCard},
  {name: 'CompareCard', Card: cards.CompareCard},
  {name: 'FreezeCallout', Card: cards.FreezeCallout},
  {name: 'PartsCard', Card: cards.PartsCard},
];

function CardSection({name, Card}: Section) {
  const {ref, progress} = useScrollProgress<HTMLElement>();
  const props = (fromExample(name) ?? fallback[name]) as BaseProps;
  return <section ref={ref} className="section" aria-label={name}>
    <div className="sticky">
      <FitCard><Card {...props} progress={progress} /></FitCard>
      <div className="label">{name} · {Math.round(progress * 100)}%</div>
    </div>
  </section>;
}

// No CardHostProvider: the default host (plain URLs and <img>) suits the browser.
export function ScrollPage() {
  return <main>{sections.map(section => <CardSection key={section.name} {...section} />)}</main>;
}
