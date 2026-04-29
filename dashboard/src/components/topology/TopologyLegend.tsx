export function TopologyLegend() {
  return (
    <div className="rounded-xl border border-white/10 bg-[#0a0a0f]/90 p-3 text-xs text-slate-300">
      <p className="mb-2 uppercase tracking-[0.2em] text-slate-500">Legend</p>
      <div className="space-y-1.5">
        <p className="flex items-center gap-2"><span className="h-3 w-3 rounded-full bg-indigo-500" /> Network Hub</p>
        <p className="flex items-center gap-2"><span className="h-2.5 w-2.5 rounded-full bg-emerald-400" /> Online Node</p>
        <p className="flex items-center gap-2"><span className="h-2.5 w-2.5 rounded-full bg-amber-400" /> Degraded Node</p>
        <p className="flex items-center gap-2"><span className="h-2.5 w-2.5 rounded-full bg-rose-400" /> Offline Node</p>
        <p className="flex items-center gap-2"><span className="inline-block h-px w-6 bg-indigo-400" /> Active Job Flow</p>
      </div>
    </div>
  );
}
