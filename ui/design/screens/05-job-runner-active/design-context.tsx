// Figma design context: node 4:18609, file 8mO8UiLLEMNEutFSkjalPD, captured 2026-10-02 via get_design_context.
// REFERENCE ONLY: Figma's React+Tailwind rendering. Rebuild in egui using ui/design/tokens.json; match screenshot.png.
// Asset URLs below are Figma temp links (expire in 7 days); local copies are in ui/design/assets/icons/ (see DESIGN.md icon table).

const assetPathPrefix = "https://www.figma.com/api/mcp/asset/b062c58f-01a5-4617-9ade-2dcba58e07a2";
const imgWindowControls = `${assetPathPrefix}/045f3.svg`;
const imgAudioLines = `${assetPathPrefix}/97823.svg`;
const imgFolderOpen = `${assetPathPrefix}/1957b.svg`;
const imgRotateCw = `${assetPathPrefix}/c25b0.svg`;
const imgSearch = `${assetPathPrefix}/690df.svg`;
const imgList = `${assetPathPrefix}/8ad19.svg`;
const imgLoaderCircle = `${assetPathPrefix}/718bb.svg`;
const imgPlay = `${assetPathPrefix}/235ba.svg`;
const imgSquare = `${assetPathPrefix}/8bf64.svg`;

export default function Component05JobRunnerActive() {
  return (
    <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex flex-col items-start overflow-clip relative rounded-[8px] size-full" data-node-id="4:18609" data-name="05 · Job runner — active">
      <div className="bg-[#272a2f] content-stretch flex h-[38px] items-center justify-between overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18610" data-name="Window title bar">
        <div className="h-[11px] relative shrink-0 w-[49px]" data-node-id="4:18611" data-name="Window controls">
          <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgWindowControls} />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:18615">
          scriptCast — ATO_EP03_five_levels
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:18616">{`⌘O  Open episode`}</p>
      </div>
      <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[22px] h-[76px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18617" data-name="Episode header">
        <div className="content-stretch flex gap-[8px] items-center overflow-clip relative shrink-0" data-node-id="4:18618" data-name="Brand">
          <div className="relative shrink-0 size-[22px]" data-node-id="4:20153" data-name="audio-lines">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgAudioLines} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:18620">
            scriptCast
          </p>
        </div>
        <div className="bg-[#363a41] h-[30px] relative shrink-0 w-px" data-node-id="4:18621" data-name="Header divider" />
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[5px] items-start min-w-px overflow-clip relative" data-node-id="4:18622" data-name="Episode identity">
          <div className="content-stretch flex items-start overflow-clip relative shrink-0" data-node-id="4:18623" data-name="Episode name">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[15px] whitespace-nowrap" data-node-id="4:18624">
              ATO_EP03_five_levels
            </p>
          </div>
          <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-pre" data-node-id="4:18625">{`Kokoro v1.0  /  48 kHz  /  392 turns  /  4 speakers  /  Rendered 03 Oct 2026, 10:32`}</p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18626" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20156" data-name="folder-open">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgFolderOpen} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18628">
            Open
          </p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18629" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20159" data-name="rotate-cw">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgRotateCw} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18631">
            Reload
          </p>
        </div>
      </div>
      <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px relative w-full" data-node-id="4:18632" data-name="Review workspace">
        <div className="bg-[#17191c] content-stretch flex flex-col gap-[6px] h-[188px] items-start overflow-clip px-[20px] py-[10px] relative shrink-0 w-full" data-node-id="4:18633" data-name="Episode timeline">
          <div className="[word-break:break-word] content-stretch flex items-center justify-between leading-[1.45] not-italic overflow-clip relative shrink-0 w-full" data-node-id="4:18634" data-name="Timeline heading">
            <div className="content-stretch flex gap-[12px] items-start overflow-clip relative shrink-0 whitespace-nowrap" data-node-id="4:18635" data-name="Timeline identity">
              <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:18636">
                Timeline
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18637">
                34:18.420 · 392 turns
              </p>
            </div>
            <div className="content-stretch flex font-['Inter:Regular'] font-normal gap-[14px] items-start overflow-clip relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18638" data-name="Zoom controls">
              <p className="relative shrink-0 whitespace-pre" data-node-id="4:18639">{`−  Zoom  +`}</p>
              <p className="relative shrink-0 whitespace-nowrap" data-node-id="4:18640">
                Fit episode
              </p>
            </div>
          </div>
          <div className="content-stretch flex gap-[2px] h-[17px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18641" data-name="Episode overview">
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px relative rounded-[1px]" data-node-id="4:18642" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18643" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18644" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18645" data-name="Overview section" />
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18646" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18647" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18648" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18649" data-name="Overview section" />
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18650" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18651" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18652" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18653" data-name="Overview section" />
            <div className="absolute border border-[#8caff3] border-solid h-[21px] left-0 rounded-[2px] top-[-2px] w-[68px]" data-node-id="4:18654" data-name="Visible range" />
          </div>
          <div className="[word-break:break-word] content-stretch flex font-['IBM_Plex_Mono:Regular'] h-[18px] items-start justify-between leading-[1.45] not-italic overflow-clip pl-[80px] relative shrink-0 text-[#717985] text-[10px] w-full whitespace-nowrap" data-node-id="4:18655" data-name="Time ruler">
            <p className="relative shrink-0" data-node-id="4:18656">
              00:00
            </p>
            <p className="relative shrink-0" data-node-id="4:18657">
              00:15
            </p>
            <p className="relative shrink-0" data-node-id="4:18658">
              00:30
            </p>
            <p className="relative shrink-0" data-node-id="4:18659">
              00:45
            </p>
            <p className="relative shrink-0" data-node-id="4:18660">
              01:00
            </p>
            <p className="relative shrink-0" data-node-id="4:18661">
              01:15
            </p>
            <p className="relative shrink-0" data-node-id="4:18662">
              01:30
            </p>
          </div>
          <div className="content-stretch flex gap-[12px] h-[92px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18663" data-name="Speaker lanes">
            <div className="content-stretch flex flex-col gap-[4px] items-start overflow-clip relative shrink-0 w-[66px]" data-node-id="4:18664" data-name="Speaker labels">
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18665" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[12px] whitespace-nowrap" data-node-id="4:18666">
                  ▎Alex
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18667" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[12px] whitespace-nowrap" data-node-id="4:18668">
                  ▎Maya
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18669" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[12px] whitespace-nowrap" data-node-id="4:18670">
                  ▎Jordan
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18671" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[12px] whitespace-nowrap" data-node-id="4:18672">
                  ▎Sam
                </p>
              </div>
            </div>
            <div className="content-stretch flex flex-[1_0_0] flex-col gap-[4px] h-[92px] items-start min-w-px overflow-clip relative" data-node-id="4:18673" data-name="Audio tracks">
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18674" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-0 overflow-clip px-[4px] right-[90.84%] rounded-[2px] top-0" data-node-id="4:18675" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18676">
                    001
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[16.02%] overflow-clip px-[4px] right-[78.07%] rounded-[2px] top-0" data-node-id="4:18677" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18678">
                    003
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[51.84%] overflow-clip px-[4px] right-[41.44%] rounded-[2px] top-0" data-node-id="4:18679" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18680">
                    008
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[79.91%] overflow-clip px-[4px] right-[10.8%] rounded-[2px] top-0" data-node-id="4:18681" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18682">
                    012
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18683" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[9.16%] overflow-clip px-[4px] right-[83.98%] rounded-[2px] top-0" data-node-id="4:18684" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18685">
                    002
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[36.91%] overflow-clip px-[4px] right-[56.69%] rounded-[2px] top-0" data-node-id="4:18686" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18687">
                    006
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[64.31%] overflow-clip px-[4px] right-[27.78%] rounded-[2px] top-0" data-node-id="4:18688" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18689">
                    010
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18690" data-name="Speaker track">
                <div className="absolute bg-[#2a3b56] border-2 border-[#8caff3] border-solid content-stretch flex h-[20px] items-center left-[21.93%] overflow-clip px-[4px] right-[68%] rounded-[2px] top-0" data-node-id="4:18691" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:18692">
                    004
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#92b7aa] border-solid content-stretch flex h-[20px] items-center left-[43.31%] overflow-clip px-[4px] right-[48.16%] rounded-[2px] top-0" data-node-id="4:18693" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:18694">
                    007
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#92b7aa] border-solid content-stretch flex h-[20px] items-center left-[72.22%] overflow-clip px-[4px] right-[20.09%] rounded-[2px] top-0" data-node-id="4:18695" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:18696">
                    011
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18697" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#c5b28d] border-solid content-stretch flex h-[20px] items-center left-[32%] overflow-clip px-[4px] right-[63.09%] rounded-[2px] top-0" data-node-id="4:18698" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[9px] whitespace-nowrap" data-node-id="4:18699">
                    005
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#c5b28d] border-solid content-stretch flex h-[20px] items-center left-[58.56%] overflow-clip px-[4px] right-[35.69%] rounded-[2px] top-0" data-node-id="4:18700" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[9px] whitespace-nowrap" data-node-id="4:18701">
                    009
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div className="bg-[#272a2f] border-[#363a41] border-b border-solid border-t content-stretch flex gap-[10px] h-[42px] items-center overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18702" data-name="Review filters">
          <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex gap-[8px] h-[27px] items-center overflow-clip px-[9px] relative rounded-[5px] shrink-0 w-[270px]" data-node-id="4:18703" data-name="Search turns">
            <div className="relative shrink-0 size-[14px]" data-node-id="4:20162" data-name="search">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgSearch} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[12px] whitespace-nowrap" data-node-id="4:18705">
              Search turns
            </p>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-nowrap" data-node-id="4:18706">
              ⌘F
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18707" data-name="Action">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18708">
              All speakers ⌄
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18709" data-name="Action">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18710">
              Failed only · 7
            </p>
          </div>
          <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-start justify-end min-w-px overflow-clip relative" data-node-id="4:18711" data-name="View mode">
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:18712">
              Turn view
            </p>
            <div className="relative shrink-0 size-[15px]" data-node-id="4:20165" data-name="list">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgList} />
            </div>
          </div>
        </div>
        <div className="content-stretch flex flex-[1_0_0] items-start min-h-px overflow-clip relative w-full" data-node-id="4:18714" data-name="Job workspace">
          <div className="[word-break:break-word] bg-[#1e2024] content-stretch flex flex-[1_0_0] flex-col h-full items-start min-w-px not-italic overflow-clip relative" data-node-id="4:18715" data-name="Turn table">
            <div className="bg-[#17191c] content-stretch flex font-['Inter:Semi_Bold'] font-semibold gap-[12px] h-[30px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 text-[#a4abb6] text-[11px] w-full" data-node-id="4:18716" data-name="Column headers">
              <p className="relative shrink-0 w-[44px]" data-node-id="4:18717">
                Turn
              </p>
              <p className="relative shrink-0 w-[92px]" data-node-id="4:18718">
                Time
              </p>
              <p className="relative shrink-0 w-[82px]" data-node-id="4:18719">
                Speaker
              </p>
              <p className="flex-[1_0_0] min-w-px relative" data-node-id="4:18720">
                First words
              </p>
              <p className="relative shrink-0 w-[60px]" data-node-id="4:18721">
                Duration
              </p>
              <p className="relative shrink-0 w-[106px]" data-node-id="4:18722">
                Verification
              </p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18723" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18724">
                001
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18725">
                00:00.000
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18726">
                ▎Alex
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18727">
                Welcome to All Things Open. Today, five levels…
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18728">
                8.24 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18729">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18730" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18731">
                002
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18732">
                00:08.240
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:18733">
                ▎Maya
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18734">
                From a single prompt to a system that can act.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18735">
                6.18 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18736">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18737" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18738">
                003
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18739">
                00:14.420
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18740">
                ▎Alex
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18741">
                Let’s start with the simplest idea: a tool.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18742">
                5.32 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18743">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#2a3b56] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18744" data-name="Turn row · selected">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#8caff3] text-[11px] w-[44px]" data-node-id="4:18745">
                004
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18746">
                00:19.740
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18747">
                ▎Jordan
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18748">
                A tool does one thing well. An agent decides…
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18749">
                9.06 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18750">{`×  Failed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18751" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18752">
                005
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18753">
                00:28.800
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#c5b28d] text-[12px] w-[82px]" data-node-id="4:18754">
                ▎Sam
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18755">
                And that distinction is easy to miss.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18756">
                4.42 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18757">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18758" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18759">
                006
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18760">
                00:33.220
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:18761">
                ▎Maya
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18762">
                So what changes when we move up a level?
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18763">
                5.76 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18764">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18765" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18766">
                007
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18767">
                00:38.980
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18768">
                ▎Jordan
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18769">
                You give the system a goal, not just a command.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18770">
                7.68 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18771">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18772" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18773">
                008
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18774">
                00:46.660
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18775">
                ▎Alex
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18776">
                That’s the shift we’ll unpack in this episode.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18777">
                6.04 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18778">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18779" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18780">
                009
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18781">
                00:52.700
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#c5b28d] text-[12px] w-[82px]" data-node-id="4:18782">
                ▎Sam
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18783">
                With a few concrete examples along the way.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18784">
                5.18 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18785">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18786" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18787">
                010
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18788">
                00:57.880
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:18789">
                ▎Maya
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18790">
                First, let’s agree on what we mean by autonomy.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18791">
                7.12 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18792">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18793" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18794">
                011
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18795">
                01:05.000
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18796">
                ▎Jordan
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18797">
                Autonomy is a spectrum, rather than a switch.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18798">
                6.92 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18799">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18800" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18801">
                012
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18802">
                01:11.920
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18803">
                ▎Alex
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18804">
                Right. The useful question is how much control…
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18805">
                8.36 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18806">{`✓  Passed`}</p>
            </div>
            <div className="content-stretch flex flex-[1_0_0] font-['Inter:Regular'] font-normal items-start justify-between leading-[1.45] min-h-[24px] overflow-clip pt-[12px] px-[16px] relative text-[#717985] text-[11px] w-full" data-node-id="4:18807" data-name="Table summary">
              <p className="relative shrink-0 whitespace-nowrap" data-node-id="4:18808">
                Showing 1–12 of 392 turns
              </p>
              <p className="relative shrink-0 whitespace-pre" data-node-id="4:18809">{`↑ ↓  Select turn`}</p>
            </div>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-l border-solid content-stretch flex flex-col h-full items-start relative shrink-0 w-[510px]" data-node-id="4:18810" data-name="Inspector">
            <div className="[word-break:break-word] bg-[#272a2f] border-[#363a41] border-b border-solid content-stretch flex h-[39px] items-center justify-between leading-[1.45] not-italic overflow-clip px-[16px] relative shrink-0 w-full whitespace-nowrap" data-node-id="4:18811" data-name="Panel heading">
              <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:18812">
                Job runner
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[10px]" data-node-id="4:18813">
                Local CLI · available
              </p>
            </div>
            <div className="content-stretch flex flex-[1_0_0] flex-col gap-[8px] items-start min-h-px overflow-clip p-[12px] relative w-full" data-node-id="4:18814" data-name="Job content">
              <div className="content-start flex flex-wrap gap-[6px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18815" data-name="Fixed job presets">
                <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18816" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18817">
                    Detect changes
                  </p>
                </div>
                <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18818" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18819">
                    Precision insert
                  </p>
                </div>
                <div className="bg-[#8caff3] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18820" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#151c29] text-[12px] whitespace-nowrap" data-node-id="4:18821">
                    Regenerate selected
                  </p>
                </div>
                <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18822" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18823">
                    Verify
                  </p>
                </div>
                <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18824" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18825">
                    Repair
                  </p>
                </div>
              </div>
              <div className="[word-break:break-word] content-stretch flex items-center justify-between not-italic overflow-clip relative shrink-0 w-full whitespace-nowrap" data-node-id="4:18826" data-name="Active job">
                <div className="content-stretch flex flex-col gap-[3px] items-start leading-[1.45] overflow-clip relative shrink-0" data-node-id="4:18827" data-name="Job identity">
                  <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[13px]" data-node-id="4:18828">
                    Regenerate selected · Turn 004
                  </p>
                  <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[10px]" data-node-id="4:18829">
                    Approved scope · audio replacement · checkpoint v03 saved
                  </p>
                </div>
                <p className="font-['Inter:Regular'] font-normal leading-[normal] relative shrink-0 text-[#8caff3] text-[11px]" data-node-id="4:18830">
                  ● Running
                </p>
              </div>
              <div className="bg-[#30343a] content-stretch flex h-[3px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18831" data-name="Job progress">
                <div className="bg-[#8caff3] h-[3px] relative shrink-0 w-[282px]" data-node-id="4:18832" data-name="Job completed progress" />
              </div>
              <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex flex-[1_0_0] flex-col gap-[5px] items-start min-h-[140px] overflow-clip p-[11px] relative rounded-[5px] w-full" data-node-id="4:18833" data-name="Streaming output">
                <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[10px] whitespace-nowrap" data-node-id="4:18834">
                  OUTPUT · last 200 lines · auto-scroll on
                </p>
                <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-pre" data-node-id="4:18835">{`10:47:02  Job accepted: regenerate turn 004`}</p>
                <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-pre" data-node-id="4:18836">{`10:47:02  Scope locked: ATO_EP03_five_levels`}</p>
                <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-pre" data-node-id="4:18837">{`10:47:03  Checkpoint saved: v03`}</p>
                <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-pre" data-node-id="4:18838">{`10:47:04  Loaded voice: Jordan / kokoro`}</p>
                <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-pre" data-node-id="4:18839">{`10:47:05  Synthesizing 9.06 s at 48 kHz`}</p>
                <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-pre" data-node-id="4:18840">{`10:47:12  Audio candidate written`}</p>
                <div className="content-stretch flex gap-[7px] items-center overflow-clip relative shrink-0" data-node-id="4:18841" data-name="Live activity">
                  <div className="relative shrink-0 size-[12px]" data-node-id="4:20168" data-name="loader-circle">
                    <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgLoaderCircle} />
                  </div>
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[1.45] not-italic relative shrink-0 text-[#8caff3] text-[11px] whitespace-nowrap" data-node-id="4:18843">
                    Verification in progress_
                  </p>
                </div>
              </div>
              <div className="content-stretch flex items-center justify-between overflow-clip relative shrink-0 w-full" data-node-id="4:18844" data-name="Job controls">
                <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:18845">
                  00:14 elapsed · reload when complete
                </p>
                <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18846" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18847">
                    Cancel
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="bg-[#272a2f] border-[#363a41] border-solid border-t content-stretch flex gap-[18px] h-[64px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18848" data-name="Playback controls">
        <div className="content-stretch flex gap-[7px] items-start overflow-clip relative shrink-0" data-node-id="4:18849" data-name="Transport">
          <div className="bg-[#8caff3] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18850" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20171" data-name="play">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgPlay} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#151c29] text-[12px] whitespace-nowrap" data-node-id="4:18852">
              Play
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18853" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20174" data-name="square">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgSquare} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18855">
              Stop
            </p>
          </div>
        </div>
        <div className="[word-break:break-word] content-stretch flex flex-col gap-[2px] items-start leading-[1.45] not-italic overflow-clip relative shrink-0 w-[226px] whitespace-nowrap" data-node-id="4:18856" data-name="Current audio">
          <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:18857">
            Turn 004 · Jordan
          </p>
          <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18858">
            00:00.000 / 00:09.060
          </p>
        </div>
        <div className="bg-[#30343a] content-stretch flex flex-[1_0_0] h-[4px] items-start min-w-px overflow-clip relative rounded-[2px]" data-node-id="4:18859" data-name="Seek track">
          <div className="bg-[#8caff3] h-[4px] relative shrink-0 w-px" data-node-id="4:18860" data-name="Elapsed progress" />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:18861">
          ☑ Auto-advance
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:18862">{`Space  Play / pause`}</p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:18863">
          MacBook Pro speakers ⌄
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#17191c] content-stretch flex font-['Inter:Regular'] font-normal h-[26px] items-center justify-between leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 text-[10px] w-full whitespace-nowrap" data-node-id="4:18864" data-name="Application status">
        <p className="relative shrink-0 text-[#8caff3]" data-node-id="4:18865">
          Regenerate selected · running · 00:14 elapsed · 1 active job
        </p>
        <p className="relative shrink-0 text-[#717985]" data-node-id="4:18866">
          Local workspace · no agent connection required
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#22262d] border-[#363a41] border-solid border-t content-stretch flex gap-[20px] h-[56px] items-center leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18867" data-name="Workshop annotation">
        <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#8caff3] text-[20px] whitespace-nowrap" data-node-id="4:18868">
          05
        </p>
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[2px] items-start min-w-px overflow-clip relative" data-node-id="4:18869" data-name="Workshop note">
          <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18870">
            A bounded job, not a blocked workspace
          </p>
          <p className="font-['Inter:Regular'] font-normal min-w-full relative shrink-0 text-[#a4abb6] text-[11px] w-[min-content]" data-node-id="4:18871">
            One safe job at a time. Episode inspection remains available; fixed presets declare scope and replacement intent before execution.
          </p>
        </div>
        <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#717985] text-[10px] whitespace-nowrap" data-node-id="4:18872">
          WORKSHOP / CORE
        </p>
      </div>
    </div>
  );
}
