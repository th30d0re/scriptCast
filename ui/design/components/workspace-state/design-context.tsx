// Figma design context for "scriptCast / Workspace state" (component set, node 4:20383)
// File 8mO8UiLLEMNEutFSkjalPD, captured 2026-10-02 via get_design_context.
//
// REFERENCE ONLY. This is Figma's React+Tailwind rendering of the design. Do not
// ship it. Rebuild it in egui: take sizes, spacing, colours, type and copy from
// here, and match screenshot.png. Icons are the local SVGs in ui/design/assets/icons/.

const imgUnplug = "../../assets/icons/unplug-22-accent.svg";
const imgTerminal = "../../assets/icons/terminal-22-danger.svg";
const imgHeadphones = "../../assets/icons/headphones-22-accent.svg";
const imgVolumeX = "../../assets/icons/volume-x-22-danger.svg";
const imgCircleAlert = "../../assets/icons/circle-alert-22-danger.svg";
const imgLoaderCircle = "../../assets/icons/loader-circle-22-accent.svg";
const imgFolderOpen = "../../assets/icons/folder-open-22-accent.svg";

type ScriptCastWorkspaceStateProps = {
  className?: string;
  state?: "Agent offline" | "CLI unavailable" | "Unavailable device" | "Missing audio" | "Error" | "Loading" | "Empty";
};

export default function ScriptCastWorkspaceState({ className, state = "Empty" }: ScriptCastWorkspaceStateProps) {
  const isAgentOffline = state === "Agent offline";
  const isCliUnavailable = state === "CLI unavailable";
  const isEmpty = state === "Empty";
  const isError = state === "Error";
  const isLoading = state === "Loading";
  const isMissingAudio = state === "Missing audio";
  const isUnavailableDevice = state === "Unavailable device";
  return (
    <div className={className || "bg-[#1e2024] border border-[#363a41] border-solid content-stretch flex flex-col gap-[18px] items-start min-h-[150px] overflow-clip p-[16px] relative rounded-[8px] w-[680px]"} id={isEmpty ? "node-4_20376" : isLoading ? "node-4_20377" : isError ? "node-4_20378" : isMissingAudio ? "node-4_20379" : isUnavailableDevice ? "node-4_20380" : isCliUnavailable ? "node-4_20381" : "node-4_20382"}>
      <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[13px] whitespace-nowrap" id={isEmpty ? "node-4_19982" : isLoading ? "node-4_19994" : isError ? "node-4_20008" : isMissingAudio ? "node-4_20022" : isUnavailableDevice ? "node-4_20034" : isCliUnavailable ? "node-4_20046" : "node-4_20058"}>
        {isLoading ? "Workspace state / Loading" : isError ? "Workspace state / Error" : isMissingAudio ? "Workspace state / Missing audio" : isUnavailableDevice ? "Workspace state / Unavailable device" : isCliUnavailable ? "Workspace state / CLI unavailable" : isAgentOffline ? "Workspace state / Agent offline" : "Workspace state / Empty"}
      </p>
      <div className="content-stretch flex flex-col gap-[14px] items-start overflow-clip relative shrink-0 w-full" id={isEmpty ? "node-4_19983" : isLoading ? "node-4_19995" : isError ? "node-4_20009" : isMissingAudio ? "node-4_20023" : isUnavailableDevice ? "node-4_20035" : isCliUnavailable ? "node-4_20047" : "node-4_20059"} data-name="Workspace state">
        <div className="content-stretch flex gap-[10px] items-center overflow-clip relative shrink-0" id={isEmpty ? "node-4_19984" : isLoading ? "node-4_19996" : isError ? "node-4_20010" : isMissingAudio ? "node-4_20024" : isUnavailableDevice ? "node-4_20036" : isCliUnavailable ? "node-4_20048" : "node-4_20060"} data-name="State title">
          {isAgentOffline && (
            <>
              <div className="relative shrink-0 size-[22px]" data-node-id="4:20333" data-name="unplug">
                <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgUnplug} />
              </div>
              <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:20062">
                Agent connection offline
              </p>
            </>
          )}
          {isCliUnavailable && (
            <>
              <div className="relative shrink-0 size-[22px]" data-node-id="4:20330" data-name="terminal">
                <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgTerminal} />
              </div>
              <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:20050">
                Rendering CLI unavailable
              </p>
            </>
          )}
          {isUnavailableDevice && (
            <>
              <div className="relative shrink-0 size-[22px]" data-node-id="4:20327" data-name="headphones">
                <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgHeadphones} />
              </div>
              <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:20038">
                Audio output unavailable
              </p>
            </>
          )}
          {isMissingAudio && (
            <>
              <div className="relative shrink-0 size-[22px]" data-node-id="4:20324" data-name="volume-x">
                <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgVolumeX} />
              </div>
              <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:20026">
                Audio missing · Turn 004
              </p>
            </>
          )}
          {isError && (
            <>
              <div className="relative shrink-0 size-[22px]" data-node-id="4:20321" data-name="circle-alert">
                <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgCircleAlert} />
              </div>
              <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:20012">
                Couldn’t open this episode
              </p>
            </>
          )}
          {isLoading && (
            <>
              <div className="relative shrink-0 size-[22px]" data-node-id="4:20318" data-name="loader-circle">
                <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgLoaderCircle} />
              </div>
              <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:19998">
                Loading ATO_EP03_five_levels…
              </p>
            </>
          )}
          {isEmpty && (
            <>
              <div className="relative shrink-0 size-[22px]" data-node-id="4:20315" data-name="folder-open">
                <img alt="" className="absolute block inset-0 max-w-none size-full" src={imgFolderOpen} />
              </div>
              <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[1.45] not-italic relative shrink-0 text-[#edf0f4] text-[18px] whitespace-nowrap" data-node-id="4:19986">
                No episode open
              </p>
            </>
          )}
        </div>
        <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] min-w-full not-italic relative shrink-0 text-[#a4abb6] text-[13px] w-[min-content]" id={isEmpty ? "node-4_19987" : isLoading ? "node-4_19999" : isError ? "node-4_20013" : isMissingAudio ? "node-4_20027" : isUnavailableDevice ? "node-4_20039" : isCliUnavailable ? "node-4_20051" : "node-4_20063"}>
          {isLoading ? "Reading turn metadata · 284 / 392 indexed. Audio playback becomes available after indexing." : isError ? "Turn metadata could not be read. Choose a rendered episode directory or fix the metadata and try again. The current episode is still open." : isMissingAudio ? "turn_004.wav was not found. Transcript and verification history are available. Restore the file or regenerate the selected turn." : isUnavailableDevice ? "The selected device was disconnected. Playback is paused at 00:04.620; your selection is unchanged. Choose another output to resume." : isCliUnavailable ? "The configured executable could not be found. Review audio normally; configure the CLI path before launching Detect changes, Precision insert, Regenerate, Verify or Repair." : isAgentOffline ? "Conversation and proposals are paused. Timeline, playback, verification and local jobs remain available. Reconnect when you’re ready." : "Open a directory containing rendered audio and metadata. Your files remain local."}
        </p>
        {["Agent offline", "CLI unavailable", "Unavailable device", "Missing audio", "Error", "Empty"].includes(state) && (
          <div className={`content-stretch flex items-start overflow-clip relative shrink-0 ${isError ? "gap-[8px]" : ""}`} id={isEmpty ? "node-4_19988" : isError ? "node-4_20014" : isMissingAudio ? "node-4_20028" : isUnavailableDevice ? "node-4_20040" : isCliUnavailable ? "node-4_20052" : "node-4_20064"} data-name="Recovery actions">
            <div className="bg-[#8caff3] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" id={isEmpty ? "node-4_19989" : isError ? "node-4_20015" : isMissingAudio ? "node-4_20029" : isUnavailableDevice ? "node-4_20041" : isCliUnavailable ? "node-4_20053" : "node-4_20065"} data-name="Action">
              <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#151c29] text-[12px] whitespace-nowrap" id={isEmpty ? "node-4_19990" : isError ? "node-4_20016" : isMissingAudio ? "node-4_20030" : isUnavailableDevice ? "node-4_20042" : isCliUnavailable ? "node-4_20054" : "node-4_20066"}>
                {isError ? "Choose directory" : isMissingAudio ? "Regenerate selected" : isUnavailableDevice ? "Choose output" : isCliUnavailable ? "Configure CLI" : isAgentOffline ? "Reconnect agent" : "Open Episode"}
              </p>
            </div>
            {isError && (
              <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:20017" data-name="Action">
                <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:20018">
                  Try again
                </p>
              </div>
            )}
          </div>
        )}
        {isLoading && (
          <>
            <div className="bg-[#30343a] content-stretch flex h-[3px] items-start overflow-clip relative shrink-0 w-full" data-node-id="4:20000" data-name="Indexing progress">
              <div className="bg-[#8caff3] h-[3px] relative shrink-0 w-[390px]" data-node-id="4:20001" data-name="Indexed progress" />
            </div>
            <div className="content-stretch flex items-start overflow-clip relative shrink-0" data-node-id="4:20002" data-name="Recovery actions">
              <div className="bg-[#30343a] border border-[#363a41] border-solid content-stretch flex h-[30px] items-center overflow-clip px-[11px] relative rounded-[5px] shrink-0" data-node-id="4:20003" data-name="Action">
                <p className="[word-break:break-word] font-['Inter:Semi_Bold'] font-semibold leading-[normal] not-italic relative shrink-0 text-[#edf0f4] text-[12px] whitespace-nowrap" data-node-id="4:20004">
                  Cancel
                </p>
              </div>
            </div>
          </>
        )}
      </div>
      <p className="[word-break:break-word] font-['Inter:Regular'] font-normal leading-[1.45] min-w-full not-italic relative shrink-0 text-[#a4abb6] text-[11px] w-[min-content]" id={isEmpty ? "node-4_19991" : isLoading ? "node-4_20005" : isError ? "node-4_20019" : isMissingAudio ? "node-4_20031" : isUnavailableDevice ? "node-4_20043" : isCliUnavailable ? "node-4_20055" : "node-4_20067"}>
        {isLoading ? "Loading is bounded and cancelable. Never manufacture a verification result." : isError ? "A malformed directory does not erase the current episode." : isMissingAudio ? "Keep metadata and all unaffected turns inspectable." : isUnavailableDevice ? "Device errors affect playback, not browsing or CLI jobs." : isCliUnavailable ? "Disable job launching, not episode review. Keep the last log and explain recovery." : isAgentOffline ? "Future agent features degrade independently. Core review and jobs still work." : "Directory-first entry; recent episodes are optional shortcuts."}
      </p>
    </div>
  );
}
