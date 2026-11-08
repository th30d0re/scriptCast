import React from 'react';
import {AbsoluteFill, useCurrentFrame} from 'remotion';
import {TokensContext} from '../tokens';
import {color, Tokens} from './theme/tokens';
import {
  ActionIcon, BootIn, CardBorder, CausalArrow, CitationCard, CompactBorder,
  DefinitionCard, DrawLine, DrawRect, EvidenceToken, FitBox, Flicker,
  GroupDirectionField, HeartIcon, KitTokens, NeuralField, NumberBadge,
  PersonIcon, PersonMark, PrecisionGauge, PredictiveUpdatePlot, QuestionHeader,
  ScanLines, SceneBorder, SignalLane, SourceLine, StageSceneFrame, StickFigure,
  SystemIcon, TermBlock, TermTriad, Trace, TrainingTextStrip, UsThemSortingField,
  WordIcon,
} from './index';

type Item = {id: string; source: string; motion?: boolean};

/** The public inventory used by the renderer and by human reviewers. */
const sheetItems: readonly [string, string, boolean?][] = [
  ['CitationCard', 'components/CitationCard.tsx'], ['DefinitionCard', 'components/DefinitionCard.tsx'],
  ['EvidenceToken', 'components/EvidenceToken.tsx'], ['GroupDirectionField', 'components/GroupDirectionField.tsx'],
  ['NeuralField', 'components/NeuralField.tsx'], ['PrecisionGauge', 'components/PrecisionGauge.tsx'],
  ['PredictiveUpdatePlot', 'components/PredictiveUpdatePlot.tsx'], ['QuestionHeader', 'components/QuestionHeader.tsx'],
  ['SignalLane', 'components/SignalLane.tsx'], ['SourceLine', 'components/SourceLine.tsx'],
  ['StageSceneFrame', 'components/StageSceneFrame.tsx'], ['TermBlock', 'components/TermBlock.tsx'],
  ['TermTriad', 'components/TermTriad.tsx'], ['TrainingTextStrip', 'components/TrainingTextStrip.tsx'], ['UsThemSortingField', 'components/UsThemSortingField.tsx'],
  ['CardBorder', 'primitives/CardBorder.tsx'], ['CompactBorder', 'primitives/CompactBorder.tsx'], ['SceneBorder', 'primitives/SceneBorder.tsx'], ['PersonMark', 'primitives/PersonMark.tsx'],
  ['NumberBadge', 'primitives/Icons.tsx'], ['PersonIcon', 'primitives/Icons.tsx'], ['ActionIcon', 'primitives/Icons.tsx'], ['SystemIcon', 'primitives/Icons.tsx'], ['WordIcon', 'primitives/Icons.tsx'], ['HeartIcon', 'primitives/Icons.tsx'], ['CausalArrow', 'primitives/Icons.tsx'], ['ScanLines', 'primitives/Icons.tsx'],
  ['BootIn', 'motion/BootIn.tsx', true], ['DrawLine', 'motion/DrawLine.tsx', true], ['DrawRect', 'motion/DrawRect.tsx', true], ['FitBox', 'motion/FitBox.tsx', true], ['Flicker', 'motion/Flicker.tsx', true], ['StickFigure', 'motion/StickFigure.tsx', true], ['Trace', 'motion/Trace.tsx', true],
];

export const KitSheetIndex: readonly Item[] = sheetItems.map(([id, source, motion]) => ({id, source, motion}));

const byId = new Map(KitSheetIndex.map((item) => [item.id, item]));
const specimenStyle: React.CSSProperties = {position: 'absolute', left: '50%', top: '47%', transform: 'translate(-50%, -50%)', width: 1200, height: 620};
const panelStyle: React.CSSProperties = {position: 'absolute', inset: 0, border: `1px solid ${color('color/line/graticule')}`, background: color('color/surface/1')};

function StaticSpecimen({id}: {id: string}) {
  const wrap = (node: React.ReactNode) => <div style={specimenStyle}>{node}</div>;
  switch (id) {
    case 'CitationCard': return wrap(<CitationCard authors="Author & Author" year={2026} title="Example work" progress={1}/>);
    case 'DefinitionCard': return wrap(<DefinitionCard term="Signal" definition="A reusable definition treatment." source="Sample source" paraphrased progress={1} accent="color/fill/gold"/>);
    case 'EvidenceToken': return wrap(<EvidenceToken label="DOCUMENTED" kind="documented" progress={1} emphasis={.4} sourceSocket={{lock: 1}} box={{x: 330, y: 270, w: 540, h: 72}}/>);
    case 'GroupDirectionField': return wrap(<GroupDirectionField box={{x: 180, y: 110, w: 840, h: 420}} marks={[{id: 'l', side: 'left', x: .4, y: .4, accent: 'ingroup'}, {id: 'r', side: 'right', x: .6, y: .55, accent: 'outgroup'}]} leftLabel="GROUP A" rightLabel="GROUP B" forwardProgress={1} reverseProgress={1}/>);
    case 'NeuralField': return wrap(<NeuralField layers={[3, 4, 2]} inputs={['A', 'B', 'C']} weights={[.2, .8, -.4]} activationProgress={1} compressionProgress={.2} labels={['OUT', 'ALT']}/>);
    case 'PrecisionGauge': return wrap(<PrecisionGauge value={.68} derived label="PRECISION" lowLabel="LOW" highLabel="HIGH"/>);
    case 'PredictiveUpdatePlot': return wrap(<PredictiveUpdatePlot evidenceMean={.34} priorMean={.7} precision={.55} showEquation termHighlight="posterior" posteriorProgress={1}/>);
    case 'QuestionHeader': return wrap(<QuestionHeader box={{x: 160, y: 180, w: 880, h: 230}} text="Which signal changes the field?" accentTerm="signal" progress={1}/>);
    case 'SignalLane': return wrap(<SignalLane box={{x: 160, y: 220, w: 880, h: 160}} label="SIGNAL" progress={1} accent="color/fill/gold" direction="right" tokens={[{id: 'a', at: .28}, {id: 'b', at: .72, shape: 'card'}]}/>);
    case 'SourceLine': return wrap(<><div style={panelStyle}/><SourceLine text="Source: example reference" x={150} y={480}/></>);
    case 'StageSceneFrame': return <StageSceneFrame segment={{instance: 'sheet', scene: 'KitSheet', start_ms: 0, end_ms: 3000, shots: []}} title="STAGE SCENE FRAME" source="Source: example reference"/>;
    case 'TermBlock': return wrap(<TermBlock term="Reusable term" summary="Compact semantic block" glyph={<PersonIcon/>} state="selected" accent="color/fill/gold" unit="UNIT 01"/>);
    case 'TermTriad': return wrap(<TermTriad boxes={[{x: 20, y: 80, w: 350, h: 300}, {x: 425, y: 80, w: 350, h: 300}, {x: 830, y: 80, w: 350, h: 300}]} terms={[{kind: 'bias', title: 'BIAS', lines: [{text: 'pattern', progress: 1}]}, {kind: 'prejudice', title: 'PREJUDICE'}, {kind: 'discrimination', title: 'ACTION'}]} states={['active', 'selected', 'active']} progress={1} bracketLabel="RELATED TERMS"/>);
    case 'TrainingTextStrip': return wrap(<TrainingTextStrip tokens={['signal', 'pattern', 'update']} patternKey="sample" progress={1} emphasis="pattern"/>);
    case 'UsThemSortingField': return wrap(<UsThemSortingField marks={[{id: 'a', x: 180, y: 130, group: 'ingroup'}, {id: 'b', x: 580, y: 260, group: 'outgroup'}]} partitionProgress={1} ingroupProgress={1} outgroupProgress={1} labels={{ingroup: 'US', outgroup: 'THEM', boundary: 'SORT'}} mode="partition"/>);
    case 'CardBorder': return <CardBorder/>;
    case 'CompactBorder': return <CompactBorder/>;
    case 'SceneBorder': return <SceneBorder/>;
    case 'PersonMark': return wrap(<PersonMark x={600} y={330} group="ingroup" emphasis={1} scale={4}/>);
    case 'NumberBadge': return wrap(<NumberBadge value="4" active={1}/>);
    case 'PersonIcon': return wrap(<PersonIcon/>);
    case 'ActionIcon': return wrap(<ActionIcon/>);
    case 'SystemIcon': return wrap(<SystemIcon/>);
    case 'WordIcon': return wrap(<WordIcon/>);
    case 'HeartIcon': return wrap(<HeartIcon/>);
    case 'CausalArrow': return wrap(<svg width="1200" height="620"><CausalArrow x1={220} x2={980} y={310}/></svg>);
    case 'ScanLines': return wrap(<svg width="1200" height="620"><ScanLines x={200} y={160} w={800} h={300} progress={1}/></svg>);
    default: return null;
  }
}

function MotionSpecimen({id, progress}: {id: string; progress: number}) {
  const common = {position: 'absolute' as const, width: 180, height: 220, top: 50};
  const render = (p: number) => {
    switch (id) {
      case 'BootIn': return <BootIn box={{x: 15, y: 25, w: 150, h: 150}} progress={p}><div style={{width: 150, height: 150, display: 'grid', placeItems: 'center', fontFamily: Tokens.fonts.mono}}>READY</div></BootIn>;
      case 'DrawLine': return <DrawLine points={[[15, 165], [65, 50], [125, 130], [165, 25]]} end="arrow" progress={p}/>;
      case 'DrawRect': return <DrawRect box={{x: 20, y: 30, w: 140, h: 135}} notch="top" progress={p}/>;
      case 'FitBox': return <FitBox text="SYSTEM" x={15} y={95} family="mono" size={18} tag="SYS·01" progress={p}/>;
      case 'Flicker': return <Flicker progress={p} seed={17}><div style={{position: 'absolute', left: 15, top: 55, width: 150, height: 100, display: 'grid', placeItems: 'center', border: `1px solid ${color('color/line/wire')}`, fontFamily: Tokens.fonts.mono}}>POWER ON</div></Flicker>;
      case 'StickFigure': return <StickFigure x={90} y={125} group="ingroup" emphasis={1} scale={1.6} pose="open" progress={p}/>;
      case 'Trace': return <svg width="180" height="220"><Trace points={[[15, 165], [65, 50], [125, 130], [165, 25]]} drawn={p} width={3}/></svg>;
      default: return null;
    }
  };
  return <div style={{...specimenStyle, width: 1120, height: 370}}>
    <div style={{...panelStyle, opacity: .6}}/>
    {[.2, .4, .6, .8, 1].map((p, index) => <div key={p} style={{...common, left: 65 + index * 220}}><div style={{position: 'absolute', top: 0, fontFamily: Tokens.fonts.mono, color: color('color/text/muted'), fontSize: 16}}>P {p.toFixed(1)}</div>{render(p)}</div>)}
    <div style={{position: 'absolute', left: 470, top: 292, width: 180, height: 40, fontFamily: Tokens.fonts.mono, color: color('color/text/accent'), fontSize: 14, textAlign: 'center'}}>RESTING STATE</div>
    <div style={{position: 'absolute', left: 470, top: 325, width: 180, height: 40}}>{render(progress)}</div>
  </div>;
}

export function KitSheet({item}: {item: string}) {
  const current = byId.get(item) ?? KitSheetIndex[0];
  const frame = useCurrentFrame();
  return <TokensContext.Provider value={KitTokens}><AbsoluteFill style={{background: color('color/bg/field'), color: color('color/text/primary'), fontFamily: Tokens.fonts.sans}}>
    <div style={{position: 'absolute', left: 96, top: 72, fontSize: 42, fontWeight: 600}}>Stage kit review sheet</div>
    <div style={{position: 'absolute', left: 98, top: 132, fontFamily: Tokens.fonts.mono, fontSize: 19, letterSpacing: 1.4, color: color('color/text/accent')}}>{current.id}</div>
    {current.motion ? <MotionSpecimen id={current.id} progress={1}/> : <StaticSpecimen id={current.id}/>} 
    <div style={{position: 'absolute', left: 98, bottom: 62, fontFamily: Tokens.fonts.mono, fontSize: 16, color: color('color/text/muted')}}>src/stage/kit/{current.source}</div>
    <div style={{position: 'absolute', right: 98, bottom: 62, fontFamily: Tokens.fonts.mono, fontSize: 16, color: color('color/text/muted')}}>FINAL · FRAME {frame}</div>
  </AbsoluteFill></TokensContext.Provider>;
}
