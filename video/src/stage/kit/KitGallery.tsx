import React from 'react';
import {TokensContext} from '../tokens';
import {
  ActionIcon, CardBorder, CitationCard, CompactBorder, DefinitionCard,
  EvidenceToken, GroupDirectionField, HeartIcon, KitTokens, MapStage, NeuralField,
  NumberBadge, PrecisionGauge, PredictiveUpdatePlot, QuestionHeader,
  ScanLines, SignalLane, StageSceneFrame, SystemIcon, TermBlock, TermTriad,
  TrainingTextStrip, UsThemSortingField, WordIcon, StickFigure, color,
} from './index';

// Each specimen is laid out at its natural size, then uniformly fitted into its cell.
function Tile({name, index, width, height, children}:React.PropsWithChildren<{name:string;index:number;width:number;height:number}>) {
  const w=548, h=266, scale=Math.min((w-32)/width,(h-58)/height,1);
  return <section data-kit-gallery={name} style={{position:'absolute',left:116+(index%3)*570,top:150+Math.floor(index/3)*280,width:w,height:h,background:color('color/surface/well'),border:`1px solid ${color('color/line/graticule')}`}}>
    <div style={{position:'absolute',left:16,top:10,font:'500 18px "IBM Plex Mono"',color:color('color/text/accent')}}>{name}</div>
    <div style={{position:'absolute',left:(w-width*scale)/2,top:44+(h-58-height*scale)/2,width,height,transform:`scale(${scale})`,transformOrigin:'top left'}}>{children}</div>
  </section>;
}

function Sheet({page, children}:React.PropsWithChildren<{page:number}>) {
  return <TokensContext.Provider value={KitTokens}>
    <StageSceneFrame segment={{instance:`gallery-${page}`,scene:`KitGallery${page}`,start_ms:0,end_ms:3000,shots:[]}} title={`STAGE KIT · YT02 / ${page} OF 2`} source="Approved components · Natural-size specimens, uniformly scaled for review">
      {children}
    </StageSceneFrame>
  </TokensContext.Provider>;
}

export function KitGallery() {
  return <Sheet page={1}>
    <Tile index={0} name="DefinitionCard" width={640} height={260}><DefinitionCard term="Term" definition="A reusable definition treatment." source="Sample source" paraphrased progress={1} accent="color/fill/gold"/></Tile>
    <Tile index={1} name="CitationCard" width={640} height={250}><CitationCard authors="Author & Author" year={2026} title="Example work" progress={1}/></Tile>
    <Tile index={2} name="PrecisionGauge" width={520} height={280}><PrecisionGauge value={.68} derived label="PRECISION" lowLabel="LOW" highLabel="HIGH"/></Tile>
    <Tile index={3} name="TermBlock" width={620} height={210}><TermBlock term="Reusable term" summary="Compact semantic block" glyph={<StickFigure x={24} y={24} group="neutral" emphasis={1} scale={0.8} animate={false}/>} state="selected" accent="color/fill/gold" unit="UNIT 01"/></Tile>
    <Tile index={4} name="Icons · NumberBadge · arrow · scan" width={500} height={180}>
      <div style={{display:'flex',justifyContent:'space-between'}}><ActionIcon/><SystemIcon/><WordIcon/><HeartIcon/><NumberBadge value="4"/></div>
      <svg width="500" height="100" style={{position:'absolute',top:80}}><ScanLines x={290} y={14} w={180} h={64} progress={1}/></svg>
    </Tile>
    <Tile index={5} name="TrainingTextStrip" width={600} height={100}><TrainingTextStrip tokens={['signal','pattern','update']} patternKey="sample" progress={1} emphasis="pattern"/></Tile>
    <Tile index={6} name="EvidenceToken · SignalLane" width={600} height={230}>
      <EvidenceToken label="DOCUMENTED" kind="documented" progress={1} emphasis={.4} sourceSocket={{lock:1}} box={{x:16,y:16,w:290,h:54}}/>
      <SignalLane box={{x:16,y:100,w:568,h:110}} label="SIGNAL" progress={1} accent="color/fill/gold" direction="right" tokens={[{id:'a',at:.28},{id:'b',at:.72,shape:'card'}]}/>
    </Tile>
    <Tile index={7} name="QuestionHeader" width={600} height={170}><QuestionHeader box={{x:12,y:12,w:576,h:146}} text="Which signal changes the field?" accentTerm="signal" progress={1}/></Tile>
    <Tile index={8} name="UsThemSortingField · PersonMark" width={762} height={432}><UsThemSortingField marks={[{id:'a',x:150,y:130,group:'ingroup'},{id:'b',x:570,y:260,group:'outgroup'}]} partitionProgress={1} ingroupProgress={1} outgroupProgress={1} labels={{ingroup:'US',outgroup:'THEM',boundary:'SORT'}} mode="partition"/></Tile>
  </Sheet>;
}

export function KitGallery2() {
  return <Sheet page={2}>
    <Tile index={0} name="NeuralField" width={922} height={462}><NeuralField layers={[3,4,2]} inputs={['A','B','C']} weights={[.2,.8,-.4]} activationProgress={1} compressionProgress={.2} labels={['OUT','ALT']}/></Tile>
    <Tile index={1} name="PredictiveUpdatePlot" width={1042} height={472}><PredictiveUpdatePlot evidenceMean={.34} priorMean={.7} precision={.55} showEquation termHighlight="posterior" posteriorProgress={1}/></Tile>
    <Tile index={2} name="TermTriad" width={1100} height={460}><TermTriad boxes={[{x:10,y:10,w:340,h:300},{x:380,y:10,w:340,h:300},{x:750,y:10,w:340,h:300}]} terms={[{kind:'bias',title:'BIAS',lines:[{text:'pattern',progress:1}]},{kind:'prejudice',title:'PREJUDICE'},{kind:'discrimination',title:'ACTION'}]} states={['active','selected','active']} progress={1} bracketLabel="RELATED TERMS"/></Tile>
    <Tile index={3} name="GroupDirectionField" width={800} height={380}><GroupDirectionField box={{x:10,y:10,w:780,h:360}} marks={[{id:'l',side:'left',x:.4,y:.4,accent:'ingroup'},{id:'r',side:'right',x:.6,y:.5,accent:'outgroup'}]} leftLabel="GROUP A" rightLabel="GROUP B" forwardProgress={1} reverseProgress={1}/></Tile>
    <Tile index={4} name="MapStage · world" width={1920} height={1080}><MapStage camera={{center:[0,0],zoom:1}} graticule/></Tile>
    <Tile index={5} name="StageSceneFrame · SourceLine" width={1920} height={1080}><StageSceneFrame segment={{instance:'sample',scene:'Sample',start_ms:0,end_ms:3000,shots:[]}} title="A scene composed from the kit" source="Source: example reference"/></Tile>
    <Tile index={6} name="CardBorder · expanded notch" width={1920} height={1080}><CardBorder/></Tile>
    <Tile index={7} name="CardBorder · collapsed notch" width={1920} height={1080}><CardBorder notch={0}/></Tile>
    <Tile index={8} name="CompactBorder" width={1920} height={1080}><CompactBorder/></Tile>
  </Sheet>;
}
