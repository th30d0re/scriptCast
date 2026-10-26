// Figma design context: node 4:18121, file 8mO8UiLLEMNEutFSkjalPD, captured 2026-10-02 via get_design_context.
// REFERENCE ONLY: Figma's React+Tailwind rendering. Rebuild in egui using ui/design/tokens.json; match screenshot.png.
// Asset URLs below are Figma temp links (expire in 7 days); local copies are in ui/design/assets/icons/ (see DESIGN.md icon table).

const assetPathPrefix = "https://www.figma.com/api/mcp/asset/4f3ec161-0b66-48f0-a514-897b42417bcf";
const imgWindowControls = `${assetPathPrefix}/045f3.svg`;
const imgAudioLines = `${assetPathPrefix}/97823.svg`;
const imgFolderOpen = `${assetPathPrefix}/1957b.svg`;
const imgRotateCw = `${assetPathPrefix}/c25b0.svg`;
const imgSearch = `${assetPathPrefix}/690df.svg`;
const imgList = `${assetPathPrefix}/8ad19.svg`;
const imgPause = `${assetPathPrefix}/72755.svg`;
const imgSquare = `${assetPathPrefix}/8bf64.svg`;

export default function Component03SelectedTurnPlaying() {
  return (
    <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex flex-col items-start overflow-clip relative rounded-[8px] size-full" data-node-id="4:18121" data-name="03 · Selected turn — playing">
      <div className="bg-[#272a2f] content-stretch flex h-[38px] items-center justify-between overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18122" data-name="Window title bar">
        <div className="h-[11px] relative shrink-0 w-[49px]" data-node-id="4:18123" data-name="Window controls">
          <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgWindowControls} />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:18127">
          scriptCast — ATO_EP03_five_levels
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:18128">{`⌘O  Open episode`}</p>
      </div>
      <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[22px] h-[76px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18129" data-name="Episode header">
        <div className="content-stretch flex gap-[8px] items-center overflow-clip relative shrink-0" data-node-id="4:18130" data-name="Brand">
          <div className="relative shrink-0 size-[22px]" data-node-id="4:20111" data-name="audio-lines">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgAudioLines} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:18132">
            scriptCast
          </p>
        </div>
        <div className="bg-[#363a41] h-[30px] relative shrink-0 w-px" data-node-id="4:18133" data-name="Header divider" />
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[5px] items-start min-w-px overflow-clip relative" data-node-id="4:18134" data-name="Episode identity">
          <div className="content-stretch flex items-start overflow-clip relative shrink-0" data-node-id="4:18135" data-name="Episode name">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[15px] whitespace-nowrap" data-node-id="4:18136">
              ATO_EP03_five_levels
            </p>
          </div>
          <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-pre" data-node-id="4:18137">{`Kokoro v1.0  /  48 kHz  /  392 turns  /  4 speakers  /  Rendered 03 Oct 2026, 10:32`}</p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18138" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20114" data-name="folder-open">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgFolderOpen} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18140">
            Open
          </p>
        </div>
        <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18141" data-name="Action">
          <div className="relative shrink-0 size-[16px]" data-node-id="4:20117" data-name="rotate-cw">
            <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgRotateCw} />
          </div>
          <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18143">
            Reload
          </p>
        </div>
      </div>
      <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px relative w-full" data-node-id="4:18144" data-name="Review workspace">
        <div className="bg-[#17191c] content-stretch flex flex-col gap-[6px] h-[188px] items-start overflow-clip px-[20px] py-[10px] relative shrink-0 w-full" data-node-id="4:18145" data-name="Episode timeline">
          <div className="[word-break:break-word] content-stretch flex items-center justify-between leading-[1.45] not-italic overflow-clip relative shrink-0 w-full" data-node-id="4:18146" data-name="Timeline heading">
            <div className="content-stretch flex gap-[12px] items-start overflow-clip relative shrink-0 whitespace-nowrap" data-node-id="4:18147" data-name="Timeline identity">
              <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:18148">
                Timeline
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18149">
                34:18.420 · 392 turns
              </p>
            </div>
            <div className="content-stretch flex font-['Inter:Regular'] font-normal gap-[14px] items-start overflow-clip relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18150" data-name="Zoom controls">
              <p className="relative shrink-0 whitespace-pre" data-node-id="4:18151">{`−  Zoom  +`}</p>
              <p className="relative shrink-0 whitespace-nowrap" data-node-id="4:18152">
                Fit episode
              </p>
            </div>
          </div>
          <div className="content-stretch flex gap-[2px] h-[17px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18153" data-name="Episode overview">
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px relative rounded-[1px]" data-node-id="4:18154" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18155" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18156" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18157" data-name="Overview section" />
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18158" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18159" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18160" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18161" data-name="Overview section" />
            <div className="bg-[#93b1d5] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18162" data-name="Overview section" />
            <div className="bg-[#c5b28d] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18163" data-name="Overview section" />
            <div className="bg-[#92b7aa] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18164" data-name="Overview section" />
            <div className="bg-[#b2a0c9] flex-[1_0_0] h-[17px] min-w-px opacity-48 relative rounded-[1px]" data-node-id="4:18165" data-name="Overview section" />
            <div className="absolute border border-[#8caff3] border-solid h-[21px] left-0 rounded-[2px] top-[-2px] w-[68px]" data-node-id="4:18166" data-name="Visible range" />
          </div>
          <div className="[word-break:break-word] content-stretch flex font-['IBM_Plex_Mono:Regular'] h-[18px] items-start justify-between leading-[1.45] not-italic overflow-clip pl-[80px] relative shrink-0 text-[#717985] text-[10px] w-full whitespace-nowrap" data-node-id="4:18167" data-name="Time ruler">
            <p className="relative shrink-0" data-node-id="4:18168">
              00:00
            </p>
            <p className="relative shrink-0" data-node-id="4:18169">
              00:15
            </p>
            <p className="relative shrink-0" data-node-id="4:18170">
              00:30
            </p>
            <p className="relative shrink-0" data-node-id="4:18171">
              00:45
            </p>
            <p className="relative shrink-0" data-node-id="4:18172">
              01:00
            </p>
            <p className="relative shrink-0" data-node-id="4:18173">
              01:15
            </p>
            <p className="relative shrink-0" data-node-id="4:18174">
              01:30
            </p>
          </div>
          <div className="content-stretch flex gap-[12px] h-[92px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18175" data-name="Speaker lanes">
            <div className="content-stretch flex flex-col gap-[4px] items-start overflow-clip relative shrink-0 w-[66px]" data-node-id="4:18176" data-name="Speaker labels">
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18177" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[12px] whitespace-nowrap" data-node-id="4:18178">
                  ▎Alex
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18179" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[12px] whitespace-nowrap" data-node-id="4:18180">
                  ▎Maya
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18181" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[12px] whitespace-nowrap" data-node-id="4:18182">
                  ▎Jordan
                </p>
              </div>
              <div className="content-stretch flex h-[20px] items-center overflow-clip relative shrink-0" data-node-id="4:18183" data-name="Lane label">
                <p className="[word-break:break-word] font-['Inter:Medium'] font-medium leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[12px] whitespace-nowrap" data-node-id="4:18184">
                  ▎Sam
                </p>
              </div>
            </div>
            <div className="content-stretch flex flex-[1_0_0] flex-col gap-[4px] h-[92px] items-start min-w-px overflow-clip relative" data-node-id="4:18185" data-name="Audio tracks">
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18186" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-0 overflow-clip px-[4px] right-[90.84%] rounded-[2px] top-0" data-node-id="4:18187" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18188">
                    001
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[16.02%] overflow-clip px-[4px] right-[78.07%] rounded-[2px] top-0" data-node-id="4:18189" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18190">
                    003
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[51.84%] overflow-clip px-[4px] right-[41.44%] rounded-[2px] top-0" data-node-id="4:18191" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18192">
                    008
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#93b1d5] border-solid content-stretch flex h-[20px] items-center left-[79.91%] overflow-clip px-[4px] right-[10.8%] rounded-[2px] top-0" data-node-id="4:18193" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#93b1d5] text-[9px] whitespace-nowrap" data-node-id="4:18194">
                    012
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18195" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[9.16%] overflow-clip px-[4px] right-[83.98%] rounded-[2px] top-0" data-node-id="4:18196" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18197">
                    002
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[36.91%] overflow-clip px-[4px] right-[56.69%] rounded-[2px] top-0" data-node-id="4:18198" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18199">
                    006
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#b2a0c9] border-solid content-stretch flex h-[20px] items-center left-[64.31%] overflow-clip px-[4px] right-[27.78%] rounded-[2px] top-0" data-node-id="4:18200" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#b2a0c9] text-[9px] whitespace-nowrap" data-node-id="4:18201">
                    010
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18202" data-name="Speaker track">
                <div className="absolute bg-[#2a3b56] border-2 border-[#8caff3] border-solid content-stretch flex h-[20px] items-center left-[21.93%] overflow-clip px-[4px] right-[68%] rounded-[2px] top-0" data-node-id="4:18203" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:18204">
                    004
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#92b7aa] border-solid content-stretch flex h-[20px] items-center left-[43.31%] overflow-clip px-[4px] right-[48.16%] rounded-[2px] top-0" data-node-id="4:18205" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:18206">
                    007
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#92b7aa] border-solid content-stretch flex h-[20px] items-center left-[72.22%] overflow-clip px-[4px] right-[20.09%] rounded-[2px] top-0" data-node-id="4:18207" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#92b7aa] text-[9px] whitespace-nowrap" data-node-id="4:18208">
                    011
                  </p>
                </div>
              </div>
              <div className="bg-[#1e2024] content-stretch flex h-[20px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:18209" data-name="Speaker track">
                <div className="absolute bg-[#2c3037] border border-[#c5b28d] border-solid content-stretch flex h-[20px] items-center left-[32%] overflow-clip px-[4px] right-[63.09%] rounded-[2px] top-0" data-node-id="4:18210" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[9px] whitespace-nowrap" data-node-id="4:18211">
                    005
                  </p>
                </div>
                <div className="absolute bg-[#2c3037] border border-[#c5b28d] border-solid content-stretch flex h-[20px] items-center left-[58.56%] overflow-clip px-[4px] right-[35.69%] rounded-[2px] top-0" data-node-id="4:18212" data-name="Turn segment">
                  <p className="[word-break:break-word] font-['IBM_Plex_Mono:Regular'] leading-[normal] not-italic relative shrink-0 text-[#c5b28d] text-[9px] whitespace-nowrap" data-node-id="4:18213">
                    009
                  </p>
                </div>
              </div>
              <div className="absolute bg-[#edf0f4] h-[92px] left-[27.07%] right-[72.85%] top-0" data-node-id="4:18214" data-name="Playhead" />
            </div>
          </div>
        </div>
        <div className="bg-[#272a2f] border-[#363a41] border-b border-solid border-t content-stretch flex gap-[10px] h-[42px] items-center overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18215" data-name="Review filters">
          <div className="bg-[#17191c] border border-[#363a41] border-solid content-stretch flex gap-[8px] h-[27px] items-center overflow-clip px-[9px] relative rounded-[5px] shrink-0 w-[270px]" data-node-id="4:18216" data-name="Search turns">
            <div className="relative shrink-0 size-[14px]" data-node-id="4:20120" data-name="search">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgSearch} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[12px] whitespace-nowrap" data-node-id="4:18218">
              Search turns
            </p>
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-nowrap" data-node-id="4:18219">
              ⌘F
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18220" data-name="Action">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18221">
              All speakers ⌄
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18222" data-name="Action">
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18223">
              Failed only · 7
            </p>
          </div>
          <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-start justify-end min-w-px overflow-clip relative" data-node-id="4:18224" data-name="View mode">
            <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:18225">
              Turn view
            </p>
            <div className="relative shrink-0 size-[15px]" data-node-id="4:20123" data-name="list">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgList} />
            </div>
          </div>
        </div>
        <div className="content-stretch flex flex-[1_0_0] items-start min-h-px relative w-full" data-node-id="4:18227" data-name="Turn review">
          <div className="[word-break:break-word] bg-[#1e2024] content-stretch flex flex-[1_0_0] flex-col h-full items-start min-w-px not-italic overflow-clip relative" data-node-id="4:18228" data-name="Turn table">
            <div className="bg-[#17191c] content-stretch flex font-['Inter:Semi_Bold'] font-semibold gap-[12px] h-[30px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 text-[#a4abb6] text-[11px] w-full" data-node-id="4:18229" data-name="Column headers">
              <p className="relative shrink-0 w-[44px]" data-node-id="4:18230">
                Turn
              </p>
              <p className="relative shrink-0 w-[92px]" data-node-id="4:18231">
                Time
              </p>
              <p className="relative shrink-0 w-[82px]" data-node-id="4:18232">
                Speaker
              </p>
              <p className="flex-[1_0_0] min-w-px relative" data-node-id="4:18233">
                First words
              </p>
              <p className="relative shrink-0 w-[60px]" data-node-id="4:18234">
                Duration
              </p>
              <p className="relative shrink-0 w-[106px]" data-node-id="4:18235">
                Verification
              </p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18236" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18237">
                001
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18238">
                00:00.000
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18239">
                ▎Alex
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18240">
                Welcome to All Things Open. Today, five levels…
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18241">
                8.24 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18242">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18243" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18244">
                002
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18245">
                00:08.240
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:18246">
                ▎Maya
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18247">
                From a single prompt to a system that can act.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18248">
                6.18 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18249">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18250" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18251">
                003
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18252">
                00:14.420
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18253">
                ▎Alex
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18254">
                Let’s start with the simplest idea: a tool.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18255">
                5.32 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18256">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#2a3b56] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18257" data-name="Turn row · playing">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#8caff3] text-[11px] w-[44px]" data-node-id="4:18258">
                ▶004
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18259">
                00:19.740
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18260">
                ▎Jordan
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18261">
                A tool does one thing well. An agent decides…
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18262">
                9.06 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18263">{`×  Failed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18264" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18265">
                005
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18266">
                00:28.800
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#c5b28d] text-[12px] w-[82px]" data-node-id="4:18267">
                ▎Sam
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18268">
                And that distinction is easy to miss.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18269">
                4.42 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18270">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18271" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18272">
                006
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18273">
                00:33.220
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:18274">
                ▎Maya
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18275">
                So what changes when we move up a level?
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18276">
                5.76 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18277">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18278" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18279">
                007
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18280">
                00:38.980
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18281">
                ▎Jordan
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18282">
                You give the system a goal, not just a command.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18283">
                7.68 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18284">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18285" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18286">
                008
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18287">
                00:46.660
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18288">
                ▎Alex
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18289">
                That’s the shift we’ll unpack in this episode.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18290">
                6.04 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18291">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18292" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18293">
                009
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18294">
                00:52.700
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#c5b28d] text-[12px] w-[82px]" data-node-id="4:18295">
                ▎Sam
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18296">
                With a few concrete examples along the way.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18297">
                5.18 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18298">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18299" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18300">
                010
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18301">
                00:57.880
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#b2a0c9] text-[12px] w-[82px]" data-node-id="4:18302">
                ▎Maya
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18303">
                First, let’s agree on what we mean by autonomy.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18304">
                7.12 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18305">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18306" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18307">
                011
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18308">
                01:05.000
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] w-[82px]" data-node-id="4:18309">
                ▎Jordan
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18310">
                Autonomy is a spectrum, rather than a switch.
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18311">
                6.92 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18312">{`✓  Passed`}</p>
            </div>
            <div className="bg-[#1e2024] border-[#363a41] border-b border-solid content-stretch flex gap-[12px] h-[28px] items-center leading-[normal] overflow-clip px-[16px] relative shrink-0 w-full" data-node-id="4:18313" data-name="Turn row">
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[44px]" data-node-id="4:18314">
                012
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[92px]" data-node-id="4:18315">
                01:11.920
              </p>
              <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#93b1d5] text-[12px] w-[82px]" data-node-id="4:18316">
                ▎Alex
              </p>
              <p className="flex-[1_0_0] font-['Inter:Regular'] font-normal min-w-px overflow-hidden relative text-[#edf0f4] text-[12px] text-ellipsis whitespace-nowrap" data-node-id="4:18317">
                Right. The useful question is how much control…
              </p>
              <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px] w-[60px]" data-node-id="4:18318">
                8.36 s
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#9abeb0] text-[11px] w-[106px] whitespace-pre-wrap" data-node-id="4:18319">{`✓  Passed`}</p>
            </div>
            <div className="content-stretch flex flex-[1_0_0] font-['Inter:Regular'] font-normal items-start justify-between leading-[1.45] min-h-[24px] overflow-clip pt-[12px] px-[16px] relative text-[#717985] text-[11px] w-full" data-node-id="4:18320" data-name="Table summary">
              <p className="relative shrink-0 whitespace-nowrap" data-node-id="4:18321">
                Showing 1–12 of 392 turns
              </p>
              <p className="relative shrink-0 whitespace-pre" data-node-id="4:18322">{`↑ ↓  Select turn`}</p>
            </div>
          </div>
          <div className="bg-[#1e2024] border-[#363a41] border-l border-solid content-stretch flex flex-col h-full items-start relative shrink-0 w-[376px]" data-node-id="4:18323" data-name="Inspector">
            <div className="[word-break:break-word] bg-[#272a2f] border-[#363a41] border-b border-solid content-stretch flex h-[39px] items-center justify-between leading-[1.45] not-italic overflow-clip px-[16px] relative shrink-0 w-full whitespace-nowrap" data-node-id="4:18324" data-name="Panel heading">
              <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:18325">
                Selected turn
              </p>
              <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#a4abb6] text-[10px]" data-node-id="4:18326">
                004 / 392
              </p>
            </div>
            <div className="content-stretch flex flex-col gap-[12px] items-start overflow-clip p-[16px] relative shrink-0 w-full" data-node-id="4:18327" data-name="Turn detail">
              <div className="[word-break:break-word] content-stretch flex items-start justify-between leading-[normal] not-italic overflow-clip relative shrink-0 w-full" data-node-id="4:18328" data-name="Turn identity">
                <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#92b7aa] text-[12px] whitespace-nowrap" data-node-id="4:18329">
                  ▎Jordan
                </p>
                <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#f08287] text-[11px] whitespace-pre" data-node-id="4:18330">{`×  Failed`}</p>
              </div>
              <div className="[word-break:break-word] content-stretch flex flex-col gap-[5px] items-start not-italic overflow-clip relative shrink-0 w-full" data-node-id="4:18331" data-name="Detail field">
                <p className="font-['Inter:Semi_Bold'] font-semibold leading-[normal] relative shrink-0 text-[#a4abb6] text-[10px] whitespace-nowrap" data-node-id="4:18332">
                  TRANSCRIPT
                </p>
                <p className="font-['Inter:Regular'] font-normal leading-[1.45] min-w-full relative shrink-0 text-[#edf0f4] text-[12px] w-[min-content]" data-node-id="4:18333">
                  A tool does one thing well. An agent decides when and how to use it.
                </p>
              </div>
              <div className="[word-break:break-word] content-stretch flex flex-col gap-[5px] items-start not-italic overflow-clip relative shrink-0 w-full" data-node-id="4:18334" data-name="Detail field">
                <p className="font-['Inter:Semi_Bold'] font-semibold leading-[normal] relative shrink-0 text-[#a4abb6] text-[10px] whitespace-nowrap" data-node-id="4:18335">
                  AUDIO
                </p>
                <p className="font-['Inter:Regular'] font-normal leading-[1.45] min-w-full relative shrink-0 text-[#edf0f4] text-[12px] w-[min-content]" data-node-id="4:18336">
                  00:19.740 → 00:28.800 · 9.060 seconds
                </p>
              </div>
              <div className="[word-break:break-word] content-stretch flex flex-col gap-[5px] items-start not-italic overflow-clip relative shrink-0 w-full" data-node-id="4:18337" data-name="Detail field">
                <p className="font-['Inter:Semi_Bold'] font-semibold leading-[normal] relative shrink-0 text-[#a4abb6] text-[10px] whitespace-nowrap" data-node-id="4:18338">
                  FILE
                </p>
                <p className="font-['Inter:Regular'] font-normal leading-[1.45] min-w-full relative shrink-0 text-[#edf0f4] text-[12px] w-[min-content]" data-node-id="4:18339">
                  turn_004.wav · mono · 48 kHz
                </p>
              </div>
              <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] min-w-full not-italic relative shrink-0 text-[#8caff3] text-[12px] w-[min-content]" data-node-id="4:18340">
                Listening to turn 004. Select any turn to switch audio. Auto-advance is on.
              </p>
              <div className="content-stretch flex gap-[8px] items-start overflow-clip relative shrink-0" data-node-id="4:18341" data-name="Turn navigation">
                <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18342" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18343">
                    ↑ Previous
                  </p>
                </div>
                <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18344" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18345">
                    ↓ Next
                  </p>
                </div>
                <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18346" data-name="Action">
                  <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18347">
                    Review failure
                  </p>
                </div>
              </div>
              <div className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[0] min-w-full not-italic relative shrink-0 text-[#717985] text-[11px] w-[min-content]" data-node-id="4:18348">
                <p className="leading-[1.45] mb-0">Space · Play / pause</p>
                <p className="leading-[1.45] mb-0">↑ ↓ · Select previous / next turn</p>
                <p className="leading-[1.45]">Esc · Stop playback</p>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="bg-[#272a2f] border-[#363a41] border-solid border-t content-stretch flex gap-[18px] h-[64px] items-center overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18349" data-name="Playback controls">
        <div className="content-stretch flex gap-[7px] items-start overflow-clip relative shrink-0" data-node-id="4:18350" data-name="Transport">
          <div className="bg-[#8caff3] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18351" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20126" data-name="pause">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgPause} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#151c29] text-[12px] whitespace-nowrap" data-node-id="4:18353">
              Pause
            </p>
          </div>
          <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex gap-[7px] h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:18354" data-name="Action">
            <div className="relative shrink-0 size-[16px]" data-node-id="4:20129" data-name="square">
              <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgSquare} />
            </div>
            <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18356">
              Stop
            </p>
          </div>
        </div>
        <div className="[word-break:break-word] content-stretch flex flex-col gap-[2px] items-start leading-[1.45] not-italic overflow-clip relative shrink-0 w-[226px] whitespace-nowrap" data-node-id="4:18357" data-name="Current audio">
          <p className="font-['Inter:Medium'] font-medium relative shrink-0 text-[#edf0f4] text-[12px]" data-node-id="4:18358">
            Turn 004 · Jordan
          </p>
          <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#a4abb6] text-[11px]" data-node-id="4:18359">
            00:04.620 / 00:09.060
          </p>
        </div>
        <div className="bg-[#30343a] content-stretch flex flex-[1_0_0] h-[4px] items-start min-w-px overflow-clip relative rounded-[2px]" data-node-id="4:18360" data-name="Seek track">
          <div className="bg-[#8caff3] h-[4px] relative shrink-0 w-[180px]" data-node-id="4:18361" data-name="Elapsed progress" />
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[12px] whitespace-nowrap" data-node-id="4:18362">
          ☑ Auto-advance
        </p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#717985] text-[11px] whitespace-pre" data-node-id="4:18363">{`Space  Play / pause`}</p>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] not-italic relative shrink-0 text-[#a4abb6] text-[11px] whitespace-nowrap" data-node-id="4:18364">
          MacBook Pro speakers ⌄
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#17191c] content-stretch flex font-['Inter:Regular'] font-normal h-[26px] items-center justify-between leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 text-[10px] w-full whitespace-nowrap" data-node-id="4:18365" data-name="Application status">
        <p className="relative shrink-0 text-[#a4abb6]" data-node-id="4:18366">
          Playing turn 004 · selected in timeline and table
        </p>
        <p className="relative shrink-0 text-[#717985]" data-node-id="4:18367">
          Local workspace · no agent connection required
        </p>
      </div>
      <div className="[word-break:break-word] bg-[#22262d] border-[#363a41] border-solid border-t content-stretch flex gap-[20px] h-[56px] items-center leading-[1.45] not-italic overflow-clip px-[20px] relative shrink-0 w-full" data-node-id="4:18368" data-name="Workshop annotation">
        <p className="font-['IBM_Plex_Mono:Regular'] relative shrink-0 text-[#8caff3] text-[20px] whitespace-nowrap" data-node-id="4:18369">
          03
        </p>
        <div className="content-stretch flex flex-[1_0_0] flex-col gap-[2px] items-start min-w-px overflow-clip relative" data-node-id="4:18370" data-name="Workshop note">
          <p className="font-['Inter:Semi_Bold'] font-semibold relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:18371">
            Listen, select, keep moving
          </p>
          <p className="font-['Inter:Regular'] font-normal min-w-full relative shrink-0 text-[#a4abb6] text-[11px] w-[min-content]" data-node-id="4:18372">
            Playback follows the selected turn. Selecting another turn switches playback immediately; auto-advance moves to the next turn when audio ends.
          </p>
        </div>
        <p className="font-['Inter:Regular'] font-normal relative shrink-0 text-[#717985] text-[10px] whitespace-nowrap" data-node-id="4:18373">
          WORKSHOP / CORE
        </p>
      </div>
    </div>
  );
}
