// Figma design context: node 4:17895, file 8mO8UiLLEMNEutFSkjalPD, captured 2026-10-02 via get_design_context.
// REFERENCE ONLY: Figma's React+Tailwind rendering. Rebuild in egui using ui/design/tokens.json; match screenshot.png.
// Asset URLs below are Figma temp links (expire in 7 days); local copies are in ui/design/assets/icons/ (see DESIGN.md icon table).

const assetPathPrefix = "https://www.figma.com/api/mcp/asset/8b785bd5-37ac-4812-8724-99aa217160df";
const imgWindowControls = `${assetPathPrefix}/045f3.svg`;
const imgAudioLines = `${assetPathPrefix}/97823.svg`;
const imgFolderOpen = `${assetPathPrefix}/1957b.svg`;
const imgRotateCw = `${assetPathPrefix}/c25b0.svg`;
const imgSearch = `${assetPathPrefix}/690df.svg`;
const imgList = `${assetPathPrefix}/8ad19.svg`;
const imgPlay = `${assetPathPrefix}/235ba.svg`;
const imgSquare = `${assetPathPrefix}/8bf64.svg`;

export default function Component02EpisodeBrowserIdle() {
  return (
    <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex flex-col items-start overflow-clip relative rounded-[8px] size-full" data-node-id="4:17895" data-name="02 · Episode browser — idle">
      <div className="bg-[#272a2f] content-stretch flex h-[38px] items-center justify-between overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:17896" data-name="Window title bar">
        <div className="h-[11px] relative shrink-0 w-[49px]" data-node-id="4:17897" data-name="Window controls">
          <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgWindowControls} />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:17901">
          scriptCast — ATO_EP03_five_levels
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:17902">{`⌘O  Open episode`}</p>
      </div>
      <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[22px] h-[76px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:17903" data-name="Episode header">
        <div className="content-stretch flex gap-[8px] items-center overflow-clip relative shrink-0" data-node-id="4:17904" data-name="Brand">
          <div className="relative shrink-0 size-[22px]" data-node-id="4:20090" data-name="audio-lines">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgAudioLines} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:17906">
            scriptCast
          </p>
        </div>
        <div className="bg-[#363a41] h-[30px] relative shrink-0 w-px" data-node-id="4:17907" data-name="Header divider" />
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[5px] items-start min-w-px overflow-clip relative" data-node-id="4:17908" data-name="Episode identity">
          <div className="content-stretch flex items-start overflow-clip relative shrink-0" data-node-id="4:17909" data-name="Episode name">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[15px] whitespace-nowrap" data-node-id="4:17910">
              ATO_EP03_five_levels
            </p>
          </div>
          <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-pre" data-node-id="4:17911">{`Kokoro v1.0  /  48 kHz  /  392 turns  /  4 speakers  /  Rendered 03 Oct 2026, 10:32`}</p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:17912" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20093" data-name="folder-open">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgFolderOpen} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:17914">
            Open
          </p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:17915" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20096" data-name="rotate-cw">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgRotateCw} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:17917">
            Reload
          </p>
        </div>
      </div>
      <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px relative w-full" data-node-id="4:17918" data-name="Review workspace">
        <div className="bg-[#17191c] content-stretch flex flex-col gap-[6px] h-[188px] items-start overflow-clip px-[20px] py-[10px] relative shrink-0 w-full" data-node-id="4:17919" data-name="Episode timeline">
          <div className="[word-break:break-word] content-stretch flex items-center justify-between leading-[1.45] not-italic overflow-clip relative shrink-0 w-full" data-node-id="4:17920" data-name="Timeline heading">
            <div className="content-stretch flex gap-[12px] items-start overflow-clip relative shrink-0 whitespace-nowrap" data-node-id="4:17921" data-name="Timeline identity">
              <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:17922">
                Timeline
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:17923">
                34:18.420 · 392 turns
              </p>
            </div>
            <div className="content-stretch flex font-['Inter:Regular'] font-normal gap-[14px] items-start overflow-clip relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:17924" data-name="Zoom controls">
              <p className="relative shrink-0 whitespace-pre" data-node-id="4:17925">{`−  Zoom  +`}</p>
              <p className="relative shrink-0 whitespace-nowrap" data-node-id="4:17926">
                Fit episode
              </p>
            </div>
          </div>
          <div className="content-stretch flex gap-[2px] h-[17px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:17927" data-name="Episode overview">
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px relative rounded-[1px]" data-node-id="4:17928" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:17929" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:17930" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:17931" data-name="Overview section" />
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:17932" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:17933" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:17934" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:17935" data-name="Overview section" />
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:17936" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:17937" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:17938" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:17939" data-name="Overview section" />
            <div className="absolute border border-[#8caff3] border-solid h-[21px] left-0 rounded-[2px] top-[-2px] w-[68px]" data-node-id="4:17940" data-name="Visible range" />
          </div>
          <div className="[word-break:break-word] content-stretch flex font-['IBM_Plex_Mono:Regular'] h-[18px] items-start justify-between leading-[1.45] not-italic overflow-clip pl-[80px] relative shrink-0 text-[#717985] text-[10px] w-full whitespace-nowrap" data-node-id="4:17941" data-name="Time ruler">
            <p className="relative shrink-0" data-node-id="4:17942">
              00:00
            </p>
            <p className="relative shrink-0" data-node-id="4:17943">
              00:15
            </p>
            <p className="relative shrink-0" data-node-id="4:17944">
              00:30
            </p>
            <p className="relative shrink-0" data-node-id="4:17945">
              00:45
            </p>
            <p className="relative shrink-0" data-node-id="4:17946">
              01:00
            </p>
            <p className="relative shrink-0" data-node-id="4:17947">
              01:15
            </p>
            <p className="relative shrink-0" data-node-id="4:17948">
              01:30
            </p>
          </div>
          <div className="content-stretch flex gap-[12px] h-[92px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:17949" data-name="Speaker lanes">
            <div className="content-stretch flex flex-col gap-[4px] items-start overflow-clip relative shrink-0 w-[66px]" data-node-id="4:17950" data-name="Speaker labels">
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:17951" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[12px] whitespace-nowrap" data-node-id="4:17952">
                  ▎Alex
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:17953" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[12px] whitespace-nowrap" data-node-id="4:17954">
                  ▎Maya
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:17955" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[12px] whitespace-nowrap" data-node-id="4:17956">
                  ▎Jordan
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:17957" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[12px] whitespace-nowrap" data-node-id="4:17958">
                  ▎Sam
                </p>
              </div>
            </div>
            <div className="content-stretch flex flex-[1_0_0] flex-col gap-[4px] h-[92px] items-start min-w-px overflow-clip relative" data-node-id="4:17959" data-name="Audio tracks">
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:17960" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-0 overflow-clip px-[4px] right-[90.84%] rounded-[2px] top-0" data-node-id="4:17961" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:17962">
                    001
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[16.02%] overflow-clip px-[4px] right-[78.07%] rounded-[2px] top-0" data-node-id="4:17963" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:17964">
                    003
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[51.84%] overflow-clip px-[4px] right-[41.44%] rounded-[2px] top-0" data-node-id="4:17965" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:17966">
                    008
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[79.91%] overflow-clip px-[4px] right-[10.8%] rounded-[2px] top-0" data-node-id="4:17967" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:17968">
                    012
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:17969" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[9.16%] overflow-clip px-[4px] right-[83.98%] rounded-[2px] top-0" data-node-id="4:17970" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:17971">
                    002
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[36.91%] overflow-clip px-[4px] right-[56.69%] rounded-[2px] top-0" data-node-id="4:17972" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:17973">
                    006
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[64.31%] overflow-clip px-[4px] right-[27.78%] rounded-[2px] top-0" data-node-id="4:17974" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:17975">
                    010
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:17976" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#92b7aa] border-solid content-stretch flex h-[20px] items-center left-[21.93%] overflow-clip px-[4px] right-[68%] rounded-[2px] top-0" data-node-id="4:17977" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:17978">
                    004
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#92b7aa] border-solid content-stretch flex h-[20px] items-center left-[43.31%] overflow-clip px-[4px] right-[48.16%] rounded-[2px] top-0" data-node-id="4:17979" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:17980">
                    007
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#92b7aa] border-solid content-stretch flex h-[20px] items-center left-[72.22%] overflow-clip px-[4px] right-[20.09%] rounded-[2px] top-0" data-node-id="4:17981" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:17982">
                    011
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:17983" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#c5b28d] border-solid content-stretch flex h-[20px] items-center left-[32%] overflow-clip px-[4px] right-[63.09%] rounded-[2px] top-0" data-node-id="4:17984" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[9px] whitespace-nowrap" data-node-id="4:17985">
                    005
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#c5b28d] border-solid content-stretch flex h-[20px] items-center left-[58.56%] overflow-clip px-[4px] right-[35.69%] rounded-[2px] top-0" data-node-id="4:17986" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[9px] whitespace-nowrap" data-node-id="4:17987">
                    009
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div className="bg-[#272a2f] border-[#363a41] border-b border-solid border-t content-stretch flex gap-[10px] h-[42px] items-center overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:17988" data-name="Review filters">
          <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex gap-[8px] h-[27px] items-center overflow-clip px-[9px] relative rounded-[5px] shrink-0 w-[270px]" data-node-id="4:17989" data-name="Search turns">
            <div className="relative shrink-0 size-[14px]" data-node-id="4:20099" data-name="search">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgSearch} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[12px] whitespace-nowrap" data-node-id="4:17991">
              Search turns
            </p>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-nowrap" data-node-id="4:17992">
              ⌘F
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:17993" data-name="Action">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:17994">
              All speakers ⌄
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:17995" data-name="Action">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:17996">
              Failed only · 7
            </p>
          </div>
          <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-start justify-end min-w-px overflow-clip relative" data-node-id="4:17997" data-name="View mode">
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:17998">
              Turn view
            </p>
            <div className="relative shrink-0 size-[15px]" data-node-id="4:20102" data-name="list">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgList} />
            </div>
          </div>
        </div>
        <div className="[word-break:break-word] bg-[#1e2024] content-stretch flex flex-[1_0_0] flex-col items-start min-h-px not-italic overflow-clip relative w-full" data-node-id="4:18000" data-name="Turn table">
          <div className="bg-[#17191c] content-stretch flex font-['Inter:Semi_Bold'] font-semibold gap-[12px] h-[30px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 text-[#a4abb6] text-[11px] w-full" data-node-id="4:18001" data-name="Column headers">
            <p className="relative shrink-0 w-[44px]" data-node-id="4:18002">
              Turn
            </p>
            <p className="relative shrink-0 w-[92px]" data-node-id="4:18003">
              Time
            </p>
            <p className="relative shrink-0 w-[82px]" data-node-id="4:18004">
              Speaker
            </p>
            <p className="flex-[1_0_0] min-w-px relative" data-node-id="4:18005">
              First words
            </p>
            <p className="relative shrink-0 w-[60px]" data-node-id="4:18006">
              Duration
            </p>
            <p className="relative shrink-0 w-[106px]" data-node-id="4:18007">
              Verification
            </p>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18008" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18009">
              001
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18010">
              00:00.000
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18011">
              ▎Alex
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18012">
              Welcome to All Things Open. Today, five levels…
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18013">
              8.24 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18014">{`✓  Passed`}</p>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18015" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18016">
              002
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18017">
              00:08.240
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:18018">
              ▎Maya
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18019">
              From a single prompt to a system that can act.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18020">
              6.18 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18021">{`✓  Passed`}</p>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18022" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18023">
              003
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18024">
              00:14.420
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18025">
              ▎Alex
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18026">
              Let’s start with the simplest idea: a tool.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18027">
              5.32 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18028">{`✓  Passed`}</p>
          </div>
          <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18029" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18030">
              004
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18031">
              00:19.740
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18032">
              ▎Jordan
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18033">
              A tool does one thing well. An agent decides…
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18034">
              9.06 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18035">{`×  Failed`}</p>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18036" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18037">
              005
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18038">
              00:28.800
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#c5b28d] text-[12px] w-[82px]" data-node-id="4:18039">
              ▎Sam
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18040">
              And that distinction is easy to miss.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18041">
              4.42 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18042">{`✓  Passed`}</p>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18043" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18044">
              006
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18045">
              00:33.220
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:18046">
              ▎Maya
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18047">
              So what changes when we move up a level?
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18048">
              5.76 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18049">{`✓  Passed`}</p>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18050" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18051">
              007
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18052">
              00:38.980
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18053">
              ▎Jordan
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18054">
              You give the system a goal, not just a command.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18055">
              7.68 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18056">{`✓  Passed`}</p>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18057" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18058">
              008
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18059">
              00:46.660
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18060">
              ▎Alex
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18061">
              That’s the shift we’ll unpack in this episode.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18062">
              6.04 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18063">{`✓  Passed`}</p>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18064" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18065">
              009
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18066">
              00:52.700
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#c5b28d] text-[12px] w-[82px]" data-node-id="4:18067">
              ▎Sam
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18068">
              With a few concrete examples along the way.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18069">
              5.18 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18070">{`✓  Passed`}</p>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18071" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18072">
              010
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18073">
              00:57.880
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:18074">
              ▎Maya
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18075">
              First, let’s agree on what we mean by autonomy.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18076">
              7.12 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18077">{`✓  Passed`}</p>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18078" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18079">
              011
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18080">
              01:05.000
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18081">
              ▎Jordan
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18082">
              Autonomy is a spectrum, rather than a switch.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18083">
              6.92 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18084">{`✓  Passed`}</p>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18085" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18086">
              012
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18087">
              01:11.920
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18088">
              ▎Alex
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18089">
              Right. The useful question is how much control…
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18090">
              8.36 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18091">{`✓  Passed`}</p>
          </div>
          <div className="content-stretch flex flex-[1_0_0] font-['Inter:Regular'] font-normal items-start justify-between leading-[1.45] min-h-[24px] overflow-clip pt-[12px] px-[16px] relative text-[#717985] text-[11px] w-full" data-node-id="4:18092" data-name="Table summary">
            <p className="relative shrink-0 whitespace-nowrap" data-node-id="4:18093">
              Showing 1–12 of 392 turns
            </p>
            <p className="relative shrink-0 whitespace-pre" data-node-id="4:18094">{`↑ ↓  Select turn`}</p>
          </div>
        </div>
      </div>
      <div className="bg-[#272a2f] border-[#363a41] border-solid border-t content-stretch flex gap-[18px] h-[64px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18095" data-name="Playback controls">
        <div className="content-stretch flex gap-[7px] items-start overflow-clip relative shrink-0" data-node-id="4:18096" data-name="Transport">
          <div className="bg-[#8caff3] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center opacity-45 overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18097" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20105" data-name="play">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgPlay} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#151c29] text-[12px] whitespace-nowrap" data-node-id="4:18099">
              Play
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center opacity-45 overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18100" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20108" data-name="square">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgSquare} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18102">
              Stop
            </p>
          </div>
        </div>
        <div className="[word-break:break-word] content-stretch flex flex-col gap-[2px] items-start leading-[1.45] not-italic overflow-clip relative shrink-0 w-[226px] whitespace-nowrap" data-node-id="4:18103" data-name="Current audio">
          <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:18104">
            Select a turn to audition
          </p>
          <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18105">
            00:00.000 / —
          </p>
        </div>
        <div className="bg-[#30343a] content-stretch flex flex-[1_0_0] h-[4px] items-start min-w-px overflow-clip relative rounded-[2px]" data-node-id="4:18106" data-name="Seek track">
          <div className="bg-[#8caff3] h-[4px] relative shrink-0 w-px" data-node-id="4:18107" data-name="Elapsed progress" />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:18108">
          ☑ Auto-advance
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:18109">{`Space  Play / pause`}</p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:18110">
          MacBook Pro speakers ⌄
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#17191c] content-stretch flex font-['Inter:Regular'] font-normal h-[26px] items-center justify-between leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 text-[10px] w-full whitespace-nowrap" data-node-id="4:18111" data-name="Application status">
        <p className="relative shrink-0 text-[#a4abb6]" data-node-id="4:18112">
          Ready · 392 turns · 385 passed · 7 failed
        </p>
        <p className="relative shrink-0 text-[#717985]" data-node-id="4:18113">
          Local workspace · no agent connection required
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#22262d] border-[#363a41] border-solid border-t content-stretch flex gap-[20px] h-[56px] items-center leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18114" data-name="Workshop annotation">
        <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#8caff3] text-[20px] whitespace-nowrap" data-node-id="4:18115">
          02
        </p>
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[2px] items-start min-w-px overflow-clip relative" data-node-id="4:18116" data-name="Workshop note">
          <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18117">
            Inspect before you listen
          </p>
          <p className="font-['Inter:Regular'] font-normal min-w-full relative shrink-0 text-[#a4abb6] text-[11px] w-[min-content]" data-node-id="4:18118">
            Speaker lanes and a keyboard-first turn table share one selection model. Filter or search locally; no agent connection is needed.
          </p>
        </div>
        <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#717985] text-[10px] whitespace-nowrap" data-node-id="4:18119">
          WORKSHOP / CORE
        </p>
      </div>
    </div>
  );
}
