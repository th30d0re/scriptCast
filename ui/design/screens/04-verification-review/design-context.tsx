// Figma design context: node 4:18375, file 8mO8UiLLEMNEutFSkjalPD, captured 2026-10-02 via get_design_context.
// REFERENCE ONLY: Figma's React+Tailwind rendering. Rebuild in egui using ui/design/tokens.json; match screenshot.png.
// Asset URLs below are Figma temp links (expire in 7 days); local copies are in ui/design/assets/icons/ (see DESIGN.md icon table).

const assetPathPrefix = "https://www.figma.com/api/mcp/asset/50e47d5f-e3b7-46c2-b792-01498e954fb9";
const imgWindowControls = `${assetPathPrefix}/045f3.svg`;
const imgAudioLines = `${assetPathPrefix}/97823.svg`;
const imgFolderOpen = `${assetPathPrefix}/1957b.svg`;
const imgRotateCw = `${assetPathPrefix}/c25b0.svg`;
const imgSearch = `${assetPathPrefix}/690df.svg`;
const imgList = `${assetPathPrefix}/8ad19.svg`;
const imgPlay = `${assetPathPrefix}/235ba.svg`;
const imgSquare = `${assetPathPrefix}/8bf64.svg`;

export default function Component04VerificationReview() {
  return (
    <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex flex-col items-start overflow-clip relative rounded-[8px] size-full" data-node-id="4:18375" data-name="04 · Verification review">
      <div className="bg-[#272a2f] content-stretch flex h-[38px] items-center justify-between overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18376" data-name="Window title bar">
        <div className="h-[11px] relative shrink-0 w-[49px]" data-node-id="4:18377" data-name="Window controls">
          <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgWindowControls} />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:18381">
          scriptCast — ATO_EP03_five_levels
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:18382">{`⌘O  Open episode`}</p>
      </div>
      <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[22px] h-[76px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18383" data-name="Episode header">
        <div className="content-stretch flex gap-[8px] items-center overflow-clip relative shrink-0" data-node-id="4:18384" data-name="Brand">
          <div className="relative shrink-0 size-[22px]" data-node-id="4:20132" data-name="audio-lines">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgAudioLines} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:18386">
            scriptCast
          </p>
        </div>
        <div className="bg-[#363a41] h-[30px] relative shrink-0 w-px" data-node-id="4:18387" data-name="Header divider" />
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[5px] items-start min-w-px overflow-clip relative" data-node-id="4:18388" data-name="Episode identity">
          <div className="content-stretch flex items-start overflow-clip relative shrink-0" data-node-id="4:18389" data-name="Episode name">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[15px] whitespace-nowrap" data-node-id="4:18390">
              ATO_EP03_five_levels
            </p>
          </div>
          <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-pre" data-node-id="4:18391">{`Kokoro v1.0  /  48 kHz  /  392 turns  /  4 speakers  /  Rendered 03 Oct 2026, 10:32`}</p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18392" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20135" data-name="folder-open">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgFolderOpen} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18394">
            Open
          </p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18395" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20138" data-name="rotate-cw">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgRotateCw} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18397">
            Reload
          </p>
        </div>
      </div>
      <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px relative w-full" data-node-id="4:18398" data-name="Review workspace">
        <div className="bg-[#17191c] content-stretch flex flex-col gap-[6px] h-[188px] items-start overflow-clip px-[20px] py-[10px] relative shrink-0 w-full" data-node-id="4:18399" data-name="Episode timeline">
          <div className="[word-break:break-word] content-stretch flex items-center justify-between leading-[1.45] not-italic overflow-clip relative shrink-0 w-full" data-node-id="4:18400" data-name="Timeline heading">
            <div className="content-stretch flex gap-[12px] items-start overflow-clip relative shrink-0 whitespace-nowrap" data-node-id="4:18401" data-name="Timeline identity">
              <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:18402">
                Timeline
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18403">
                34:18.420 · 392 turns
              </p>
            </div>
            <div className="content-stretch flex font-['Inter:Regular'] font-normal gap-[14px] items-start overflow-clip relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18404" data-name="Zoom controls">
              <p className="relative shrink-0 whitespace-pre" data-node-id="4:18405">{`−  Zoom  +`}</p>
              <p className="relative shrink-0 whitespace-nowrap" data-node-id="4:18406">
                Fit episode
              </p>
            </div>
          </div>
          <div className="content-stretch flex gap-[2px] h-[17px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18407" data-name="Episode overview">
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px relative rounded-[1px]" data-node-id="4:18408" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18409" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18410" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18411" data-name="Overview section" />
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18412" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18413" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18414" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18415" data-name="Overview section" />
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18416" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18417" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18418" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18419" data-name="Overview section" />
            <div className="absolute bg-[#f08287] h-[17px] left-[0.96%] right-[98.79%] top-0" data-node-id="4:18420" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[9.94%] right-[89.81%] top-0" data-node-id="4:18421" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[22.67%] right-[77.08%] top-0" data-node-id="4:18422" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[32.45%] right-[67.3%] top-0" data-node-id="4:18423" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[52.1%] right-[47.65%] top-0" data-node-id="4:18424" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[69.05%] right-[30.7%] top-0" data-node-id="4:18425" data-name="Failed turn marker" />
            <div className="absolute bg-[#f08287] h-[17px] left-[87.66%] right-[12.09%] top-0" data-node-id="4:18426" data-name="Failed turn marker" />
            <div className="absolute border border-[#8caff3] border-solid h-[21px] left-0 rounded-[2px] top-[-2px] w-[68px]" data-node-id="4:18427" data-name="Visible range" />
          </div>
          <div className="[word-break:break-word] content-stretch flex font-['IBM_Plex_Mono:Regular'] h-[18px] items-start justify-between leading-[1.45] not-italic overflow-clip pl-[80px] relative shrink-0 text-[#717985] text-[10px] w-full whitespace-nowrap" data-node-id="4:18428" data-name="Time ruler">
            <p className="relative shrink-0" data-node-id="4:18429">
              00:00
            </p>
            <p className="relative shrink-0" data-node-id="4:18430">
              00:15
            </p>
            <p className="relative shrink-0" data-node-id="4:18431">
              00:30
            </p>
            <p className="relative shrink-0" data-node-id="4:18432">
              00:45
            </p>
            <p className="relative shrink-0" data-node-id="4:18433">
              01:00
            </p>
            <p className="relative shrink-0" data-node-id="4:18434">
              01:15
            </p>
            <p className="relative shrink-0" data-node-id="4:18435">
              01:30
            </p>
          </div>
          <div className="content-stretch flex gap-[12px] h-[92px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18436" data-name="Speaker lanes">
            <div className="content-stretch flex flex-col gap-[4px] items-start overflow-clip relative shrink-0 w-[66px]" data-node-id="4:18437" data-name="Speaker labels">
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18438" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[12px] whitespace-nowrap" data-node-id="4:18439">
                  ▎Alex
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18440" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[12px] whitespace-nowrap" data-node-id="4:18441">
                  ▎Maya
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18442" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[12px] whitespace-nowrap" data-node-id="4:18443">
                  ▎Jordan
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18444" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[12px] whitespace-nowrap" data-node-id="4:18445">
                  ▎Sam
                </p>
              </div>
            </div>
            <div className="content-stretch flex flex-[1_0_0] flex-col gap-[4px] h-[92px] items-start min-w-px overflow-clip relative" data-node-id="4:18446" data-name="Audio tracks">
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18447" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-0 overflow-clip px-[4px] right-[90.84%] rounded-[2px] top-0" data-node-id="4:18448" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18449">
                    001
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[16.02%] overflow-clip px-[4px] right-[78.07%] rounded-[2px] top-0" data-node-id="4:18450" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18451">
                    003
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[51.84%] overflow-clip px-[4px] right-[41.44%] rounded-[2px] top-0" data-node-id="4:18452" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18453">
                    008
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[79.91%] overflow-clip px-[4px] right-[10.8%] rounded-[2px] top-0" data-node-id="4:18454" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18455">
                    012
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18456" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[9.16%] overflow-clip px-[4px] right-[83.98%] rounded-[2px] top-0" data-node-id="4:18457" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18458">
                    002
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[36.91%] overflow-clip px-[4px] right-[56.69%] rounded-[2px] top-0" data-node-id="4:18459" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18460">
                    006
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[64.31%] overflow-clip px-[4px] right-[27.78%] rounded-[2px] top-0" data-node-id="4:18461" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18462">
                    010
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18463" data-name="Speaker track">
                <div className="absolute bg-[#40282d] border-2 border-[#8caff3] border-solid content-stretch flex h-[20px] items-center left-[21.93%] overflow-clip px-[4px] right-[68%] rounded-[2px] top-0" data-node-id="4:18464" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#f08287] text-[9px] whitespace-nowrap" data-node-id="4:18465">
                    004
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#92b7aa] border-solid content-stretch flex h-[20px] items-center left-[43.31%] overflow-clip px-[4px] right-[48.16%] rounded-[2px] top-0" data-node-id="4:18466" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:18467">
                    007
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#92b7aa] border-solid content-stretch flex h-[20px] items-center left-[72.22%] overflow-clip px-[4px] right-[20.09%] rounded-[2px] top-0" data-node-id="4:18468" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:18469">
                    011
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18470" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#c5b28d] border-solid content-stretch flex h-[20px] items-center left-[32%] overflow-clip px-[4px] right-[63.09%] rounded-[2px] top-0" data-node-id="4:18471" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[9px] whitespace-nowrap" data-node-id="4:18472">
                    005
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#c5b28d] border-solid content-stretch flex h-[20px] items-center left-[58.56%] overflow-clip px-[4px] right-[35.69%] rounded-[2px] top-0" data-node-id="4:18473" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[9px] whitespace-nowrap" data-node-id="4:18474">
                    009
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div className="bg-[#272a2f] border-[#363a41] border-b border-solid border-t content-stretch flex gap-[10px] h-[42px] items-center overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18475" data-name="Review filters">
          <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex gap-[8px] h-[27px] items-center overflow-clip px-[9px] relative rounded-[5px] shrink-0 w-[270px]" data-node-id="4:18476" data-name="Search turns">
            <div className="relative shrink-0 size-[14px]" data-node-id="4:20141" data-name="search">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgSearch} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[12px] whitespace-nowrap" data-node-id="4:18478">
              Search turns
            </p>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-nowrap" data-node-id="4:18479">
              ⌘F
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18480" data-name="Action">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18481">
              All speakers ⌄
            </p>
          </div>
          <div className="bg-[#40282d] border border-[#f08287] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18482" data-name="Action">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#f08287] text-[12px] whitespace-nowrap" data-node-id="4:18483">
              ✓ Failed only · 7
            </p>
          </div>
          <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-start justify-end min-w-px overflow-clip relative" data-node-id="4:18484" data-name="View mode">
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:18485">
              Turn view
            </p>
            <div className="relative shrink-0 size-[15px]" data-node-id="4:20144" data-name="list">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgList} />
            </div>
          </div>
        </div>
        <div className="content-stretch flex flex-[1_0_0] items-start min-h-px overflow-clip relative w-full" data-node-id="4:18487" data-name="Failure review">
          <div className="[word-break:break-word] bg-[#1e2024] content-stretch flex flex-[1_0_0] flex-col h-full items-start min-w-px not-italic overflow-clip relative" data-node-id="4:18488" data-name="Turn table">
            <div className="bg-[#17191c] content-stretch flex font-['Inter:Semi_Bold'] font-semibold gap-[12px] h-[30px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 text-[#a4abb6] text-[11px] w-full" data-node-id="4:18489" data-name="Column headers">
              <p className="relative shrink-0 w-[44px]" data-node-id="4:18490">
                Turn
              </p>
              <p className="relative shrink-0 w-[92px]" data-node-id="4:18491">
                Time
              </p>
              <p className="relative shrink-0 w-[82px]" data-node-id="4:18492">
                Speaker
              </p>
              <p className="flex-[1_0_0] min-w-px relative" data-node-id="4:18493">
                First words
              </p>
              <p className="relative shrink-0 w-[60px]" data-node-id="4:18494">
                Duration
              </p>
              <p className="relative shrink-0 w-[106px]" data-node-id="4:18495">
                Verification
              </p>
            </div>
            <div className="bg-[#2a3b56] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18496" data-name="Turn row · selected">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#8caff3] text-[11px] w-[44px]" data-node-id="4:18497">
                004
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18498">
                00:19.740
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18499">
                ▎Jordan
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18500">
                A tool does one thing well. An agent decides…
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18501">
                9.06 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18502">{`×  Failed`}</p>
            </div>
            <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18503" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18504">
                037
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18505">
                03:24.680
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:18506">
                ▎Maya
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18507">
                The second level introduces a planning step.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18508">
                6.52 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18509">{`×  Failed`}</p>
            </div>
            <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18510" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18511">
                089
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18512">
                07:46.220
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18513">
                ▎Jordan
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18514">
                The feedback loop is what makes it adaptive.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18515">
                8.40 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18516">{`×  Failed`}</p>
            </div>
            <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18517" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18518">
                126
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18519">
                11:08.060
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#c5b28d] text-[12px] w-[82px]" data-node-id="4:18520">
                ▎Sam
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18521">
                A human still owns the decision to deploy.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18522">
                5.26 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18523">{`×  Failed`}</p>
            </div>
            <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18524" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18525">
                203
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18526">
                17:52.440
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18527">
                ▎Alex
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18528">
                Let’s put those five levels into perspective.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18529">
                7.88 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18530">{`×  Failed`}</p>
            </div>
            <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18531" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18532">
                271
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18533">
                23:40.820
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:18534">
                ▎Maya
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18535">
                Not every workflow needs an autonomous agent.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18536">
                6.34 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18537">{`×  Failed`}</p>
            </div>
            <div className="bg-[#2c2227] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18538" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18539">
                344
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18540">
                30:04.160
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18541">
                ▎Jordan
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18542">
                Start with the least complexity that works.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18543">
                5.72 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18544">{`×  Failed`}</p>
            </div>
            <div className="content-stretch flex flex-[1_0_0] font-['Inter:Regular'] font-normal items-start justify-between leading-[1.45] min-h-[24px] overflow-clip pt-[12px] px-[16px] relative text-[#717985] text-[11px] w-full whitespace-nowrap" data-node-id="4:18545" data-name="Table summary">
              <p className="relative shrink-0" data-node-id="4:18546">
                7 failures · 385 passed · no missing audio
              </p>
              <p className="relative shrink-0" data-node-id="4:18547">
                Last verified 10:42
              </p>
            </div>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-l border-solid content-stretch flex flex-col h-full items-start relative shrink-0 w-[436px]" data-node-id="4:18548" data-name="Inspector">
            <div className="[word-break:break-word] bg-[#272a2f] border-[#363a41] border-b border-solid content-stretch flex h-[39px] items-center justify-between leading-[1.45] not-italic overflow-clip px-[16px] relative shrink-0 w-full whitespace-nowrap" data-node-id="4:18549" data-name="Panel heading">
              <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:18550">
                Verification · Turn 004
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[10px]" data-node-id="4:18551">
                Jordan
              </p>
            </div>
            <div className="content-stretch flex flex-col gap-[11px] items-start overflow-clip p-[16px] relative shrink-0 w-full" data-node-id="4:18552" data-name="Verification details">
              <div className="[word-break:break-word] content-stretch flex items-start justify-between not-italic overflow-clip relative shrink-0 text-[#f08287] w-full" data-node-id="4:18553" data-name="Verification result">
                <p className="font-['Inter:Regular'] font-normal leading-[normal] relative shrink-0 text-[11px] whitespace-pre" data-node-id="4:18554">{`×  Failed`}</p>
                <p className="font-['IBM_Plex_Mono:Regular'] leading-[1.45] relative shrink-0 text-[12px] whitespace-nowrap" data-node-id="4:18555">
                  Score 0.71 / 0.90
                </p>
              </div>
              <div className="[word-break:break-word] content-stretch flex flex-col gap-[5px] items-start not-italic overflow-clip relative shrink-0 w-full" data-node-id="4:18556" data-name="Detail field">
                <p className="font-['Inter:Semi_Bold'] font-semibold leading-[normal] relative shrink-0 text-[#a4abb6] text-[10px] whitespace-nowrap" data-node-id="4:18557">
                  EXPECTED TEXT
                </p>
                <p className="font-['Inter:Regular'] font-normal leading-[1.45] min-w-full relative shrink-0 text-[#edf0f4] text-[12px] w-[min-content]" data-node-id="4:18558">
                  A tool does one thing well. An agent decides when and how to use it.
                </p>
              </div>
              <div className="[word-break:break-word] content-stretch flex flex-col gap-[5px] items-start not-italic overflow-clip relative shrink-0 w-full" data-node-id="4:18559" data-name="Detail field">
                <p className="font-['Inter:Semi_Bold'] font-semibold leading-[normal] relative shrink-0 text-[#a4abb6] text-[10px] whitespace-nowrap" data-node-id="4:18560">
                  HEARD TEXT
                </p>
                <p className="font-['Inter:Regular'] font-normal leading-[1.45] min-w-full relative shrink-0 text-[#f08287] text-[12px] w-[min-content]" data-node-id="4:18561">
                  A tool does one thing well. An agent decides when to use it.
                </p>
              </div>
              <div className="[word-break:break-word] border-[#363a41] border-b border-solid border-t content-stretch flex flex-col gap-[5px] items-start leading-[1.45] not-italic overflow-clip py-[8px] relative shrink-0 w-full whitespace-nowrap" data-node-id="4:18562" data-name="Stress check verdicts">
                <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#a4abb6] text-[10px]" data-node-id="4:18563">
                  STRESS CHECKS
                </p>
                <div className="content-stretch flex font-['Inter:Regular'] font-normal items-start justify-between overflow-clip relative shrink-0 text-[11px] w-full" data-node-id="4:18564" data-name="Stress verdict">
                  <p className="relative shrink-0 text-[#a4abb6]" data-node-id="4:18565">
                    Text alignment
                  </p>
                  <p className="relative shrink-0 text-[#f08287]" data-node-id="4:18566">
                    Fail · “and how” omitted
                  </p>
                </div>
                <div className="content-stretch flex font-['Inter:Regular'] font-normal items-start justify-between overflow-clip relative shrink-0 text-[11px] w-full" data-node-id="4:18567" data-name="Stress verdict">
                  <p className="relative shrink-0 text-[#a4abb6]" data-node-id="4:18568">
                    Emphasis / stress
                  </p>
                  <p className="relative shrink-0 text-[#f08287]" data-node-id="4:18569">
                    Fail · “decides” flattened
                  </p>
                </div>
                <div className="content-stretch flex font-['Inter:Regular'] font-normal items-start justify-between overflow-clip relative shrink-0 text-[11px] w-full" data-node-id="4:18570" data-name="Stress verdict">
                  <p className="relative shrink-0 text-[#a4abb6]" data-node-id="4:18571">
                    Clipping / silence
                  </p>
                  <p className="relative shrink-0 text-[#9abeb0]" data-node-id="4:18572">
                    Pass · clean boundaries
                  </p>
                </div>
              </div>
              <div className="content-stretch flex gap-[8px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18573" data-name="Synthesis actions">
                <div className="bg-[#8caff3] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18574" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#151c29] text-[12px] whitespace-nowrap" data-node-id="4:18575">
                    Regenerate selected
                  </p>
                </div>
                <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18576" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18577">
                    Repair
                  </p>
                </div>
              </div>
              <div className="content-stretch flex gap-[12px] items-center overflow-clip relative shrink-0 w-full" data-node-id="4:18578" data-name="Verification action">
                <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18579" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18580">
                    Verify again
                  </p>
                </div>
                <p className="[word-break:break-word] flex-[1_0_0] font-['Inter:Regular'] font-normal leading-[1.45] min-w-px not-italic relative text-[#a4abb6] text-[10px]" data-node-id="4:18581">
                  Replaces verification output only
                </p>
              </div>
              <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[10px] w-full" data-node-id="4:18582">
                Regenerate / Repair can replace audio. Confirm scope before launching a job.
              </p>
            </div>
          </div>
        </div>
      </div>
      <div className="bg-[#272a2f] border-[#363a41] border-solid border-t content-stretch flex gap-[18px] h-[64px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18583" data-name="Playback controls">
        <div className="content-stretch flex gap-[7px] items-start overflow-clip relative shrink-0" data-node-id="4:18584" data-name="Transport">
          <div className="bg-[#8caff3] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18585" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20147" data-name="play">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgPlay} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#151c29] text-[12px] whitespace-nowrap" data-node-id="4:18587">
              Play
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18588" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20150" data-name="square">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgSquare} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18590">
              Stop
            </p>
          </div>
        </div>
        <div className="[word-break:break-word] content-stretch flex flex-col gap-[2px] items-start leading-[1.45] not-italic overflow-clip relative shrink-0 w-[226px] whitespace-nowrap" data-node-id="4:18591" data-name="Current audio">
          <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:18592">
            Turn 004 · Jordan
          </p>
          <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18593">
            00:00.000 / 00:09.060
          </p>
        </div>
        <div className="bg-[#30343a] content-stretch flex flex-[1_0_0] h-[4px] items-start min-w-px overflow-clip relative rounded-[2px]" data-node-id="4:18594" data-name="Seek track">
          <div className="bg-[#8caff3] h-[4px] relative shrink-0 w-px" data-node-id="4:18595" data-name="Elapsed progress" />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:18596">
          ☑ Auto-advance
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:18597">{`Space  Play / pause`}</p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:18598">
          MacBook Pro speakers ⌄
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#17191c] content-stretch flex font-['Inter:Regular'] font-normal h-[26px] items-center justify-between leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 text-[10px] w-full whitespace-nowrap" data-node-id="4:18599" data-name="Application status">
        <p className="relative shrink-0 text-[#a4abb6]" data-node-id="4:18600">
          Verification review · 7 failed turns · threshold 0.90
        </p>
        <p className="relative shrink-0 text-[#717985]" data-node-id="4:18601">
          Local workspace · no agent connection required
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#22262d] border-[#363a41] border-solid border-t content-stretch flex gap-[20px] h-[56px] items-center leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18602" data-name="Workshop annotation">
        <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#8caff3] text-[20px] whitespace-nowrap" data-node-id="4:18603">
          04
        </p>
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[2px] items-start min-w-px overflow-clip relative" data-node-id="4:18604" data-name="Workshop note">
          <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18605">
            Diagnose before replacing
          </p>
          <p className="font-['Inter:Regular'] font-normal min-w-full relative shrink-0 text-[#a4abb6] text-[11px] w-[min-content]" data-node-id="4:18606">
            Failures use red consistently. Synthesis and replacement are separated from inspection; regenerate, repair and re-verification require confirmation.
          </p>
        </div>
        <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#717985] text-[10px] whitespace-nowrap" data-node-id="4:18607">
          WORKSHOP / CORE
        </p>
      </div>
    </div>
  );
}
