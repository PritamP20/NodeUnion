import { estimateCost, type EstimatorInput, ESTIMATOR_PRESETS } from "@/lib/pricing";

type Props = {
  onSelect: (next: EstimatorInput) => void;
};

export function PresetCards({ onSelect }: Props) {
  return (
    <section className="glass-card rounded-[1.75rem] p-5 sm:p-6">
      <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-300">Compare Plans</p>
      <div className="mt-4 grid gap-3 md:grid-cols-3">
        {ESTIMATOR_PRESETS.map((preset) => {
          const estimate = estimateCost({ ...preset, includeBaseFee: true });

          return (
            <button
              type="button"
              key={preset.key}
              onClick={() => onSelect({ ...preset, includeBaseFee: true })}
              className="rounded-xl border border-white/10 bg-white/5 p-4 text-left transition hover:border-indigo-400/40 hover:bg-indigo-500/10"
            >
              <p className="text-sm font-semibold text-slate-100">{preset.label}</p>
              <p className="mt-2 text-xs text-slate-400">{preset.cpuCores} CPU / {preset.ramGb}GB RAM</p>
              <p className="mt-1 text-xs text-slate-400">{preset.hours}h x {preset.jobCount} jobs</p>
              <p className="mt-3 font-mono text-sm text-indigo-300">{estimate.totalSol.toFixed(4)} SOL</p>
            </button>
          );
        })}
      </div>
    </section>
  );
}
