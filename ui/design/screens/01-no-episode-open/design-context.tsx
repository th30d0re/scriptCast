// Figma design context: node 4:17810, file 8mO8UiLLEMNEutFSkjalPD, captured 2026-10-02 via get_design_context.
// REFERENCE ONLY: Figma's React+Tailwind rendering. Rebuild in egui using ui/design/tokens.json; match screenshot.png.
// Asset URLs below are Figma temp links (expire in 7 days); local copies are in ui/design/assets/icons/ (see DESIGN.md icon table).

const assetPathPrefix = "https://www.figma.com/api/mcp/asset/299be544-8f8e-449e-8bcd-234ec1411694";
const imgWindowControls = `${assetPathPrefix}/045f3.svg`;
const imgAudioLines = `${assetPathPrefix}/97823.svg`;
const imgFolderOpen = `${assetPathPrefix}/1957b.svg`;
const imgRotateCw = `${assetPathPrefix}/c25b0.svg`;
const imgFolderOpen1 = `${assetPathPrefix}/05ef7.svg`;
const imgFolderOpen2 = `${assetPathPrefix}/63bb0.svg`;
const imgPlay = `${assetPathPrefix}/235ba.svg`;
const imgSquare = `${assetPathPrefix}/8bf64.svg`;

export default function Component01NoEpisodeOpen() {
  return (
    <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex flex-col items-start overflow-clip relative rounded-[8px] size-full" data-node-id="4:17810" data-name="01 · No episode open">
      <div className="bg-[#272a2f] content-stretch flex h-[38px] items-center justify-between overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:17811" data-name="Window title bar">
        <div className="h-[11px] relative shrink-0 w-[49px]" data-node-id="4:17812" data-name="Window controls">
          <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgWindowControls} />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:17816">
          scriptCast — Episode workspace
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:17817">{`⌘O  Open episode`}</p>
      </div>
      <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[22px] h-[76px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:17818" data-name="Episode header">
        <div className="content-stretch flex gap-[8px] items-center overflow-clip relative shrink-0" data-node-id="4:17819" data-name="Brand">
          <div className="relative shrink-0 size-[22px]" data-node-id="4:20069" data-name="audio-lines">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgAudioLines} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:17821">
            scriptCast
          </p>
        </div>
        <div className="bg-[#363a41] h-[30px] relative shrink-0 w-px" data-node-id="4:17822" data-name="Header divider" />
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[5px] items-start min-w-px overflow-clip relative" data-node-id="4:17823" data-name="Episode identity">
          <div className="content-stretch flex items-start overflow-clip relative shrink-0" data-node-id="4:17824" data-name="Episode name">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[15px] whitespace-nowrap" data-node-id="4:17825">
              No episode open
            </p>
          </div>
          <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:17826">
            Open a rendered episode directory to begin
          </p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:17827" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20072" data-name="folder-open">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgFolderOpen} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:17829">
            Open
          </p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center opacity-45 overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:17830" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20075" data-name="rotate-cw">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgRotateCw} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:17832">
            Reload
          </p>
        </div>
      </div>
      <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px relative w-full" data-node-id="4:17833" data-name="Review workspace">
        <div className="content-stretch flex flex-[1_0_0] gap-[64px] items-center justify-center min-h-px overflow-clip relative w-full" data-node-id="4:17834" data-name="Empty workspace">
          <div className="content-stretch flex flex-col gap-[20px] items-start overflow-clip relative shrink-0 w-[410px]" data-node-id="4:17835" data-name="Open episode guidance">
            <div className="relative shrink-0 size-[42px]" data-node-id="4:20078" data-name="folder-open">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgFolderOpen1} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] min-w-full not-italic relative shrink-0 text-[#edf0f4] text-[30px] w-[min-content]" data-node-id="4:17837">
              Your episode starts here.
            </p>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] min-w-full not-italic relative shrink-0 text-[#a4abb6] text-[14px] w-[min-content]" data-node-id="4:17838">
              Open an episode directory containing rendered audio and metadata. scriptCast loads the timeline, speakers, turns, and verification results in one local workspace.
            </p>
            <div className="content-stretch flex gap-[12px] items-center overflow-clip relative shrink-0" data-node-id="4:17839" data-name="Open actions">
              <div className="bg-[#8caff3] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:17840" data-name="Action">
                <div className="relative shrink-0 size-[16px]" data-node-id="4:20081" data-name="folder-open">
                  <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgFolderOpen2} />
                </div>
                <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#151c29] text-[12px] whitespace-nowrap" data-node-id="4:17842">
                  Open Episode
                </p>
              </div>
              <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:17843">
                ⌘O
              </p>
            </div>
            <div className="[word-break:break-word] content-stretch flex flex-col gap-[7px] items-start leading-[1.45] not-italic overflow-clip pt-[16px] relative shrink-0 w-full" data-node-id="4:17844" data-name="Expected directory contents">
              <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#717985] text-[10px] whitespace-nowrap" data-node-id="4:17845">
                EXPECTED DIRECTORY CONTENTS
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:17846">
                episode/
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[12px] whitespace-pre" data-node-id="4:17847">{`  rendered audio  +  turn metadata`}</p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[12px] whitespace-pre" data-node-id="4:17848">{`  speaker mapping + verification report`}</p>
            </div>
          </div>
          <div className="bg-[#363a41] h-[350px] relative shrink-0 w-px" data-node-id="4:17849" data-name="Content divider" />
          <div className="[word-break:break-word] content-stretch flex flex-col gap-[8px] items-start not-italic overflow-clip relative shrink-0 w-[386px] whitespace-nowrap" data-node-id="4:17850" data-name="Recent episodes">
            <div className="content-stretch flex items-start justify-between leading-[1.45] overflow-clip pb-[12px] relative shrink-0 w-full" data-node-id="4:17851" data-name="Recent heading">
              <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[13px]" data-node-id="4:17852">
                Recent episodes
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#717985] text-[11px]" data-node-id="4:17853">
                Open directory ↗
              </p>
            </div>
            <div className="border-[#363a41] border-b border-solid content-stretch flex flex-col gap-[5px] items-start leading-[1.45] overflow-clip py-[15px] relative shrink-0 w-full" data-node-id="4:17854" data-name="Recent episode">
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#edf0f4] text-[14px]" data-node-id="4:17855">
                ATO_EP03_five_levels
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[12px]" data-node-id="4:17856">
                Today, 10:32 · 392 turns · 34:18
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#717985] text-[10px]" data-node-id="4:17857">
                ~/Podcasts/ATO/EP03
              </p>
            </div>
            <div className="border-[#363a41] border-b border-solid content-stretch flex flex-col gap-[5px] items-start leading-[1.45] overflow-clip py-[15px] relative shrink-0 w-full" data-node-id="4:17858" data-name="Recent episode">
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#edf0f4] text-[14px]" data-node-id="4:17859">
                ATO_EP02_tools_to_agents
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[12px]" data-node-id="4:17860">
                Yesterday, 16:08 · 318 turns · 28:42
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#717985] text-[10px]" data-node-id="4:17861">
                ~/Podcasts/ATO/EP02
              </p>
            </div>
            <div className="border-[#363a41] border-b border-solid content-stretch flex flex-col gap-[5px] items-start leading-[1.45] overflow-clip py-[15px] relative shrink-0 w-full" data-node-id="4:17862" data-name="Recent episode">
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#edf0f4] text-[14px]" data-node-id="4:17863">
                ATO_EP01_the_new_workflow
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[12px]" data-node-id="4:17864">
                01 Oct, 14:20 · 286 turns · 25:06
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#717985] text-[10px]" data-node-id="4:17865">
                ~/Podcasts/ATO/EP01
              </p>
            </div>
            <div className="content-stretch flex font-['Inter:Regular'] font-normal gap-[8px] items-start overflow-clip pt-[14px] relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:17866" data-name="Local privacy">
              <p className="leading-[normal] relative shrink-0" data-node-id="4:17867">
                ● Local files
              </p>
              <p className="leading-[1.45] relative shrink-0" data-node-id="4:17868">
                Nothing uploaded. No connection required.
              </p>
            </div>
          </div>
        </div>
      </div>
      <div className="bg-[#272a2f] border-[#363a41] border-solid border-t content-stretch flex gap-[18px] h-[64px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:17869" data-name="Playback controls">
        <div className="content-stretch flex gap-[7px] items-start overflow-clip relative shrink-0" data-node-id="4:17870" data-name="Transport">
          <div className="bg-[#8caff3] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center opacity-45 overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:17871" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20084" data-name="play">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgPlay} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#151c29] text-[12px] whitespace-nowrap" data-node-id="4:17873">
              Play
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center opacity-45 overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:17874" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20087" data-name="square">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgSquare} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:17876">
              Stop
            </p>
          </div>
        </div>
        <div className="[word-break:break-word] content-stretch flex flex-col gap-[2px] items-start leading-[1.45] not-italic overflow-clip relative shrink-0 w-[226px] whitespace-nowrap" data-node-id="4:17877" data-name="Current audio">
          <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:17878">
            Select a turn to audition
          </p>
          <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:17879">
            00:00.000 / —
          </p>
        </div>
        <div className="bg-[#30343a] content-stretch flex flex-[1_0_0] h-[4px] items-start min-w-px overflow-clip relative rounded-[2px]" data-node-id="4:17880" data-name="Seek track">
          <div className="bg-[#8caff3] h-[4px] relative shrink-0 w-px" data-node-id="4:17881" data-name="Elapsed progress" />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:17882">
          ☑ Auto-advance
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:17883">{`Space  Play / pause`}</p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:17884">
          MacBook Pro speakers ⌄
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#17191c] content-stretch flex font-['Inter:Regular'] font-normal h-[26px] items-center justify-between leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 text-[10px] w-full whitespace-nowrap" data-node-id="4:17885" data-name="Application status">
        <p className="relative shrink-0 text-[#a4abb6]" data-node-id="4:17886">
          No episode open · CLI available
        </p>
        <p className="relative shrink-0 text-[#717985]" data-node-id="4:17887">
          Local workspace · no agent connection required
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#22262d] border-[#363a41] border-solid border-t content-stretch flex gap-[20px] h-[56px] items-center leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:17888" data-name="Workshop annotation">
        <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#8caff3] text-[20px] whitespace-nowrap" data-node-id="4:17889">
          01
        </p>
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[2px] items-start min-w-px overflow-clip relative" data-node-id="4:17890" data-name="Workshop note">
          <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:17891">
            Open a rendered episode
          </p>
          <p className="font-['Inter:Regular'] font-normal min-w-full relative shrink-0 text-[#a4abb6] text-[11px] w-[min-content]" data-node-id="4:17892">
            Directory-first entry. Recent episodes are shortcuts, not a separate library. Audio review and CLI jobs do not need an agent.
          </p>
        </div>
        <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#717985] text-[10px] whitespace-nowrap" data-node-id="4:17893">
          WORKSHOP / CORE
        </p>
      </div>
    </div>
  );
}
