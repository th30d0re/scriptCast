// Figma design context: node 4:18874, file 8mO8UiLLEMNEutFSkjalPD, captured 2026-10-02 via get_design_context.
// REFERENCE ONLY: Figma's React+Tailwind rendering. Rebuild in egui using ui/design/tokens.json; match screenshot.png.
// Asset URLs below are Figma temp links (expire in 7 days); local copies are in ui/design/assets/icons/ (see DESIGN.md icon table).

const assetPathPrefix = "https://www.figma.com/api/mcp/asset/0a98f6e4-39ab-4f93-9498-7a08d7a4d755";
const imgWindowControls = `${assetPathPrefix}/045f3.svg`;
const imgAudioLines = `${assetPathPrefix}/97823.svg`;
const imgFolderOpen = `${assetPathPrefix}/1957b.svg`;
const imgRotateCw = `${assetPathPrefix}/c25b0.svg`;
const imgSearch = `${assetPathPrefix}/690df.svg`;
const imgList = `${assetPathPrefix}/8ad19.svg`;
const imgShieldCheck = `${assetPathPrefix}/3db3f.svg`;
const imgTriangleAlert = `${assetPathPrefix}/2141c.svg`;
const imgPlay = `${assetPathPrefix}/235ba.svg`;
const imgSquare = `${assetPathPrefix}/8bf64.svg`;

export default function Component06ConfirmAudioReplacement() {
  return (
    <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex flex-col items-start overflow-clip relative rounded-[8px] size-full" data-node-id="4:18874" data-name="06 · Confirm audio replacement">
      <div className="bg-[#272a2f] content-stretch flex h-[38px] items-center justify-between overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18875" data-name="Window title bar">
        <div className="h-[11px] relative shrink-0 w-[49px]" data-node-id="4:18876" data-name="Window controls">
          <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgWindowControls} />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:18880">
          scriptCast — ATO_EP03_five_levels
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:18881">{`⌘O  Open episode`}</p>
      </div>
      <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[22px] h-[76px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18882" data-name="Episode header">
        <div className="content-stretch flex gap-[8px] items-center overflow-clip relative shrink-0" data-node-id="4:18883" data-name="Brand">
          <div className="relative shrink-0 size-[22px]" data-node-id="4:20177" data-name="audio-lines">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgAudioLines} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:18885">
            scriptCast
          </p>
        </div>
        <div className="bg-[#363a41] h-[30px] relative shrink-0 w-px" data-node-id="4:18886" data-name="Header divider" />
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[5px] items-start min-w-px overflow-clip relative" data-node-id="4:18887" data-name="Episode identity">
          <div className="content-stretch flex items-start overflow-clip relative shrink-0" data-node-id="4:18888" data-name="Episode name">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[15px] whitespace-nowrap" data-node-id="4:18889">
              ATO_EP03_five_levels
            </p>
          </div>
          <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-pre" data-node-id="4:18890">{`Kokoro v1.0  /  48 kHz  /  392 turns  /  4 speakers  /  Rendered 03 Oct 2026, 10:32`}</p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18891" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20180" data-name="folder-open">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgFolderOpen} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18893">
            Open
          </p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18894" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20183" data-name="rotate-cw">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgRotateCw} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18896">
            Reload
          </p>
        </div>
      </div>
      <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px relative w-full" data-node-id="4:18897" data-name="Review workspace">
        <div className="bg-[#17191c] content-stretch flex flex-col gap-[6px] h-[188px] items-start overflow-clip px-[20px] py-[10px] relative shrink-0 w-full" data-node-id="4:18898" data-name="Episode timeline">
          <div className="[word-break:break-word] content-stretch flex items-center justify-between leading-[1.45] not-italic overflow-clip relative shrink-0 w-full" data-node-id="4:18899" data-name="Timeline heading">
            <div className="content-stretch flex gap-[12px] items-start overflow-clip relative shrink-0 whitespace-nowrap" data-node-id="4:18900" data-name="Timeline identity">
              <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:18901">
                Timeline
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18902">
                34:18.420 · 392 turns
              </p>
            </div>
            <div className="content-stretch flex font-['Inter:Regular'] font-normal gap-[14px] items-start overflow-clip relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18903" data-name="Zoom controls">
              <p className="relative shrink-0 whitespace-pre" data-node-id="4:18904">{`−  Zoom  +`}</p>
              <p className="relative shrink-0 whitespace-nowrap" data-node-id="4:18905">
                Fit episode
              </p>
            </div>
          </div>
          <div className="content-stretch flex gap-[2px] h-[17px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18906" data-name="Episode overview">
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px relative rounded-[1px]" data-node-id="4:18907" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18908" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18909" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18910" data-name="Overview section" />
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18911" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18912" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18913" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18914" data-name="Overview section" />
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18915" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18916" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18917" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18918" data-name="Overview section" />
            <div className="absolute bg-[#f08287] h-[17px] left-[0.96%] right-[98.79%] top-0" data-node-id="4:18919" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[9.94%] right-[89.81%] top-0" data-node-id="4:18920" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[22.67%] right-[77.08%] top-0" data-node-id="4:18921" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[32.45%] right-[67.3%] top-0" data-node-id="4:18922" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[52.1%] right-[47.65%] top-0" data-node-id="4:18923" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[69.05%] right-[30.7%] top-0" data-node-id="4:18924" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[87.66%] right-[12.09%] top-0" data-node-id="4:18925" data-name="Failed turn marker" />
            <div className="absolute border border-[#8caff3] border-solid h-[21px] left-0 rounded-[2px] top-[-2px] w-[68px]" data-node-id="4:18926" data-name="Visible range" />
          </div>
          <div className="[word-break:break-word] content-stretch flex font-['IBM_Plex_Mono:Regular'] h-[18px] items-start justify-between leading-[1.45] not-italic overflow-clip pl-[80px] relative shrink-0 text-[#717985] text-[10px] w-full whitespace-nowrap" data-node-id="4:18927" data-name="Time ruler">
            <p className="relative shrink-0" data-node-id="4:18928">
              00:00
            </p>
            <p className="relative shrink-0" data-node-id="4:18929">
              00:15
            </p>
            <p className="relative shrink-0" data-node-id="4:18930">
              00:30
            </p>
            <p className="relative shrink-0" data-node-id="4:18931">
              00:45
            </p>
            <p className="relative shrink-0" data-node-id="4:18932">
              01:00
            </p>
            <p className="relative shrink-0" data-node-id="4:18933">
              01:15
            </p>
            <p className="relative shrink-0" data-node-id="4:18934">
              01:30
            </p>
          </div>
          <div className="content-stretch flex gap-[12px] h-[92px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18935" data-name="Speaker lanes">
            <div className="content-stretch flex flex-col gap-[4px] items-start overflow-clip relative shrink-0 w-[66px]" data-node-id="4:18936" data-name="Speaker labels">
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18937" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[12px] whitespace-nowrap" data-node-id="4:18938">
                  ▎Alex
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18939" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[12px] whitespace-nowrap" data-node-id="4:18940">
                  ▎Maya
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18941" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[12px] whitespace-nowrap" data-node-id="4:18942">
                  ▎Jordan
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18943" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[12px] whitespace-nowrap" data-node-id="4:18944">
                  ▎Sam
                </p>
              </div>
            </div>
            <div className="content-stretch flex flex-[1_0_0] flex-col gap-[4px] h-[92px] items-start min-w-px overflow-clip relative" data-node-id="4:18945" data-name="Audio tracks">
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18946" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-0 overflow-clip px-[4px] right-[90.84%] rounded-[2px] top-0" data-node-id="4:18947" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18948">
                    001
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[16.02%] overflow-clip px-[4px] right-[78.07%] rounded-[2px] top-0" data-node-id="4:18949" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18950">
                    003
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[51.84%] overflow-clip px-[4px] right-[41.44%] rounded-[2px] top-0" data-node-id="4:18951" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18952">
                    008
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[79.91%] overflow-clip px-[4px] right-[10.8%] rounded-[2px] top-0" data-node-id="4:18953" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18954">
                    012
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18955" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[9.16%] overflow-clip px-[4px] right-[83.98%] rounded-[2px] top-0" data-node-id="4:18956" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18957">
                    002
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[36.91%] overflow-clip px-[4px] right-[56.69%] rounded-[2px] top-0" data-node-id="4:18958" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18959">
                    006
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[64.31%] overflow-clip px-[4px] right-[27.78%] rounded-[2px] top-0" data-node-id="4:18960" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18961">
                    010
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18962" data-name="Speaker track">
                <div className="absolute bg-[#40282d] border-2 border-[#8caff3] border-solid content-stretch flex h-[20px] items-center left-[21.93%] overflow-clip px-[4px] right-[68%] rounded-[2px] top-0" data-node-id="4:18963" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#f08287] text-[9px] whitespace-nowrap" data-node-id="4:18964">
                    004
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#92b7aa] border-solid content-stretch flex h-[20px] items-center left-[43.31%] overflow-clip px-[4px] right-[48.16%] rounded-[2px] top-0" data-node-id="4:18965" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:18966">
                    007
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#92b7aa] border-solid content-stretch flex h-[20px] items-center left-[72.22%] overflow-clip px-[4px] right-[20.09%] rounded-[2px] top-0" data-node-id="4:18967" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:18968">
                    011
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18969" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#c5b28d] border-solid content-stretch flex h-[20px] items-center left-[32%] overflow-clip px-[4px] right-[63.09%] rounded-[2px] top-0" data-node-id="4:18970" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[9px] whitespace-nowrap" data-node-id="4:18971">
                    005
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#c5b28d] border-solid content-stretch flex h-[20px] items-center left-[58.56%] overflow-clip px-[4px] right-[35.69%] rounded-[2px] top-0" data-node-id="4:18972" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[9px] whitespace-nowrap" data-node-id="4:18973">
                    009
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div className="bg-[#272a2f] border-[#363a41] border-b border-solid border-t content-stretch flex gap-[10px] h-[42px] items-center overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18974" data-name="Review filters">
          <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex gap-[8px] h-[27px] items-center overflow-clip px-[9px] relative rounded-[5px] shrink-0 w-[270px]" data-node-id="4:18975" data-name="Search turns">
            <div className="relative shrink-0 size-[14px]" data-node-id="4:20186" data-name="search">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgSearch} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[12px] whitespace-nowrap" data-node-id="4:18977">
              Search turns
            </p>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-nowrap" data-node-id="4:18978">
              ⌘F
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18979" data-name="Action">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18980">
              All speakers ⌄
            </p>
          </div>
          <div className="bg-[#40282d] border border-[#f08287] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18981" data-name="Action">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#f08287] text-[12px] whitespace-nowrap" data-node-id="4:18982">
              ✓ Failed only · 7
            </p>
          </div>
          <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-start justify-end min-w-px overflow-clip relative" data-node-id="4:18983" data-name="View mode">
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:18984">
              Turn view
            </p>
            <div className="relative shrink-0 size-[15px]" data-node-id="4:20189" data-name="list">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgList} />
            </div>
          </div>
        </div>
        <div className="[word-break:break-word] bg-[#1e2024] content-stretch flex flex-[1_0_0] flex-col items-start min-h-px not-italic overflow-clip relative w-full" data-node-id="4:18986" data-name="Turn table">
          <div className="bg-[#17191c] content-stretch flex font-['Inter:Semi_Bold'] font-semibold gap-[12px] h-[30px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 text-[#a4abb6] text-[11px] w-full" data-node-id="4:18987" data-name="Column headers">
            <p className="relative shrink-0 w-[44px]" data-node-id="4:18988">
              Turn
            </p>
            <p className="relative shrink-0 w-[92px]" data-node-id="4:18989">
              Time
            </p>
            <p className="relative shrink-0 w-[82px]" data-node-id="4:18990">
              Speaker
            </p>
            <p className="flex-[1_0_0] min-w-px relative" data-node-id="4:18991">
              First words
            </p>
            <p className="relative shrink-0 w-[60px]" data-node-id="4:18992">
              Duration
            </p>
            <p className="relative shrink-0 w-[106px]" data-node-id="4:18993">
              Verification
            </p>
          </div>
          <div className="bg-[#2a3b56] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18994" data-name="Turn row · selected">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#8caff3] text-[11px] w-[44px]" data-node-id="4:18995">
              004
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18996">
              00:19.740
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18997">
              ▎Jordan
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18998">
              A tool does one thing well. An agent decides…
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18999">
              9.06 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:19000">{`×  Failed`}</p>
          </div>
          <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:19001" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:19002">
              037
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:19003">
              03:24.680
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:19004">
              ▎Maya
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:19005">
              The second level introduces a planning step.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:19006">
              6.52 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:19007">{`×  Failed`}</p>
          </div>
          <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:19008" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:19009">
              089
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:19010">
              07:46.220
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:19011">
              ▎Jordan
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:19012">
              The feedback loop is what makes it adaptive.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:19013">
              8.40 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:19014">{`×  Failed`}</p>
          </div>
          <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:19015" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:19016">
              126
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:19017">
              11:08.060
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#c5b28d] text-[12px] w-[82px]" data-node-id="4:19018">
              ▎Sam
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:19019">
              A human still owns the decision to deploy.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:19020">
              5.26 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:19021">{`×  Failed`}</p>
          </div>
          <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:19022" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:19023">
              203
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:19024">
              17:52.440
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:19025">
              ▎Alex
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:19026">
              Let’s put those five levels into perspective.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:19027">
              7.88 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:19028">{`×  Failed`}</p>
          </div>
          <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:19029" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:19030">
              271
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:19031">
              23:40.820
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:19032">
              ▎Maya
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:19033">
              Not every workflow needs an autonomous agent.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:19034">
              6.34 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:19035">{`×  Failed`}</p>
          </div>
          <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:19036" data-name="Turn row">
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:19037">
              344
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:19038">
              30:04.160
            </p>
            <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:19039">
              ▎Jordan
            </p>
            <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:19040">
              Start with the least complexity that works.
            </p>
            <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:19041">
              5.72 s
            </p>
            <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:19042">{`×  Failed`}</p>
          </div>
          <div className="content-stretch flex flex-[1_0_0] font-['Inter:Regular'] font-normal items-start justify-between leading-[1.45] min-h-[24px] overflow-clip pt-[12px] px-[16px] relative text-[#717985] text-[11px] w-full whitespace-nowrap" data-node-id="4:19043" data-name="Table summary">
            <p className="relative shrink-0" data-node-id="4:19044">
              7 failures · 385 passed · no missing audio
            </p>
            <p className="relative shrink-0" data-node-id="4:19045">
              Last verified 10:42
            </p>
          </div>
        </div>
        <div className="absolute bg-[rgba(16,18,22,0.79)] inset-0" data-node-id="4:19046" data-name="Modal scrim" />
        <div className="-translate-x-1/2 -translate-y-1/2 absolute content-stretch flex flex-col items-start left-1/2 top-1/2 w-[540px]" data-node-id="4:19047" data-name="Confirmation sheet">
          <div className="bg-[#272a2f] border border-[#363a41] border-solid content-stretch flex flex-col gap-[18px] items-start overflow-clip p-[24px] relative rounded-[12px] shadow-[0px_16px_48px_0px_rgba(0,0,0,0.38)] shrink-0 w-[540px]" data-node-id="4:19048" data-name="Confirmation dialog">
            <div className="content-stretch flex gap-[12px] items-center overflow-clip relative shrink-0" data-node-id="4:19049" data-name="Confirmation heading">
              <div className="relative shrink-0 size-[26px]" data-node-id="4:20192" data-name="shield-check">
                <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgShieldCheck} />
              </div>
              <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[20px] whitespace-nowrap" data-node-id="4:19051">
                Regenerate selected turn?
              </p>
            </div>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] min-w-full not-italic relative shrink-0 text-[#a4abb6] text-[13px] w-[min-content]" data-node-id="4:19052">
              Synthesize new audio for the selected turn and replace its rendered audio. Other turns will not change.
            </p>
            <div className="[word-break:break-word] bg-[#17191c] content-stretch flex flex-col gap-[10px] items-start not-italic overflow-clip p-[14px] relative rounded-[5px] shrink-0 w-full" data-node-id="4:19053" data-name="Affected scope">
              <div className="content-stretch flex flex-col gap-[5px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:19054" data-name="Detail field">
                <p className="font-['Inter:Semi_Bold'] font-semibold leading-[normal] relative shrink-0 text-[#a4abb6] text-[10px] whitespace-nowrap" data-node-id="4:19055">
                  EPISODE
                </p>
                <p className="font-['Inter:Regular'] font-normal leading-[1.45] min-w-full relative shrink-0 text-[#edf0f4] text-[12px] w-[min-content]" data-node-id="4:19056">
                  ATO_EP03_five_levels
                </p>
              </div>
              <div className="content-stretch flex flex-col gap-[5px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:19057" data-name="Detail field">
                <p className="font-['Inter:Semi_Bold'] font-semibold leading-[normal] relative shrink-0 text-[#a4abb6] text-[10px] whitespace-nowrap" data-node-id="4:19058">
                  AFFECTED TURNS
                </p>
                <p className="font-['Inter:Regular'] font-normal leading-[1.45] min-w-full relative shrink-0 text-[#edf0f4] text-[12px] w-[min-content]" data-node-id="4:19059">
                  004 · Jordan · 00:19.740–00:28.800
                </p>
              </div>
            </div>
            <div className="content-stretch flex gap-[9px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:19060" data-name="Safety warning">
              <div className="relative shrink-0 size-[16px]" data-node-id="4:20195" data-name="triangle-alert">
                <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgTriangleAlert} />
              </div>
              <p className="[word-break:break-word] flex-[1_0_0] font-['Inter:Regular'] font-normal leading-[1.45] min-w-px not-italic relative text-[#f08287] text-[12px]" data-node-id="4:19062">
                Existing audio for turn 004 will be replaced. A checkpoint is saved first.
              </p>
            </div>
            <div className="[word-break:break-word] bg-[#1e2024] border border-[#363a41] border-solid content-stretch flex flex-col gap-[8px] items-start not-italic overflow-clip p-[12px] relative rounded-[5px] shrink-0 w-full" data-node-id="4:19063" data-name="Command disclosure">
              <p className="font-['Inter:Semi_Bold'] font-semibold leading-[1.45] relative shrink-0 text-[#edf0f4] text-[11px] whitespace-nowrap" data-node-id="4:19064">
                ⌄ Command summary · illustrative only
              </p>
              <div className="font-['IBM_Plex_Mono:Regular'] leading-[0] min-w-full relative shrink-0 text-[#a4abb6] text-[11px] w-[min-content] whitespace-pre-wrap" data-node-id="4:19065">
                <p className="leading-[1.45] mb-0">scriptcast render --episode ATO_EP03_five_levels</p>
                <p className="leading-[1.45]">{`  --turn 004 --replace-audio`}</p>
              </div>
              <p className="font-['Inter:Regular'] font-normal leading-[1.45] min-w-full relative shrink-0 text-[#717985] text-[10px] w-[min-content]" data-node-id="4:19066">
                Concept command, not verified production CLI syntax.
              </p>
            </div>
            <div className="content-stretch flex gap-[9px] items-start justify-end overflow-clip relative shrink-0 w-full" data-node-id="4:19067" data-name="Dialog actions">
              <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:19068" data-name="Action">
                <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:19069">
                  Cancel
                </p>
              </div>
              <div className="bg-[#8caff3] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:19070" data-name="Action">
                <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#151c29] text-[12px] whitespace-nowrap" data-node-id="4:19071">
                  Confirm · Regenerate selected
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="bg-[#272a2f] border-[#363a41] border-solid border-t content-stretch flex gap-[18px] h-[64px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:19072" data-name="Playback controls">
        <div className="content-stretch flex gap-[7px] items-start overflow-clip relative shrink-0" data-node-id="4:19073" data-name="Transport">
          <div className="bg-[#8caff3] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:19074" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20198" data-name="play">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgPlay} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#151c29] text-[12px] whitespace-nowrap" data-node-id="4:19076">
              Play
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:19077" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20201" data-name="square">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgSquare} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:19079">
              Stop
            </p>
          </div>
        </div>
        <div className="[word-break:break-word] content-stretch flex flex-col gap-[2px] items-start leading-[1.45] not-italic overflow-clip relative shrink-0 w-[226px] whitespace-nowrap" data-node-id="4:19080" data-name="Current audio">
          <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:19081">
            Turn 004 · Jordan
          </p>
          <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:19082">
            00:00.000 / 00:09.060
          </p>
        </div>
        <div className="bg-[#30343a] content-stretch flex flex-[1_0_0] h-[4px] items-start min-w-px overflow-clip relative rounded-[2px]" data-node-id="4:19083" data-name="Seek track">
          <div className="bg-[#8caff3] h-[4px] relative shrink-0 w-px" data-node-id="4:19084" data-name="Elapsed progress" />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:19085">
          ☑ Auto-advance
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:19086">{`Space  Play / pause`}</p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:19087">
          MacBook Pro speakers ⌄
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#17191c] content-stretch flex font-['Inter:Regular'] font-normal h-[26px] items-center justify-between leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 text-[10px] w-full whitespace-nowrap" data-node-id="4:19088" data-name="Application status">
        <p className="relative shrink-0 text-[#a4abb6]" data-node-id="4:19089">
          Awaiting confirmation · no job has started
        </p>
        <p className="relative shrink-0 text-[#717985]" data-node-id="4:19090">
          Local workspace · no agent connection required
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#22262d] border-[#363a41] border-solid border-t content-stretch flex gap-[20px] h-[56px] items-center leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:19091" data-name="Workshop annotation">
        <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#8caff3] text-[20px] whitespace-nowrap" data-node-id="4:19092">
          06
        </p>
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[2px] items-start min-w-px overflow-clip relative" data-node-id="4:19093" data-name="Workshop note">
          <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:19094">
            Confirm exact scope and consequence
          </p>
          <p className="font-['Inter:Regular'] font-normal min-w-full relative shrink-0 text-[#a4abb6] text-[11px] w-[min-content]" data-node-id="4:19095">
            A destructive operation never starts silently. Episode, turns and replacement intent are explicit; the disclosed CLI command is illustrative, not production syntax.
          </p>
        </div>
        <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#717985] text-[10px] whitespace-nowrap" data-node-id="4:19096">
          WORKSHOP / CORE
        </p>
      </div>
    </div>
  );
}
