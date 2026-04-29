type Props = {
  showJobFlow: boolean;
  autoLayout: boolean;
  onToggleJobFlow: () => void;
  onToggleAutoLayout: () => void;
  onResetView: () => void;
};

export function TopologyControls({
  showJobFlow,
  autoLayout,
  onToggleJobFlow,
  onToggleAutoLayout,
  onResetView,
}: Props) {
  return (
    <div className="flex flex-col items-end gap-2 rounded-xl border border-white/10 bg-[#0a0a0f]/90 p-3 text-xs text-slate-300">
      <button type="button" onClick={onToggleJobFlow} className="rounded-md border border-white/10 bg-white/5 px-2 py-1 hover:bg-white/10">
        {showJobFlow ? "Hide" : "Show"} job flow
      </button>
      <button type="button" onClick={onToggleAutoLayout} className="rounded-md border border-white/10 bg-white/5 px-2 py-1 hover:bg-white/10">
        Auto-layout: {autoLayout ? "On" : "Off"}
      </button>
      <button type="button" onClick={onResetView} className="rounded-md border border-white/10 bg-white/5 px-2 py-1 hover:bg-white/10">
        Reset view
      </button>
    </div>
  );
}
