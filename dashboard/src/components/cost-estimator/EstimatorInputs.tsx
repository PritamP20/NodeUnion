import type { EstimatorInput } from "@/lib/pricing";

type Props = {
  value: EstimatorInput;
  onChange: (next: EstimatorInput) => void;
};

export function EstimatorInputs({ value, onChange }: Props) {
  return (
    <section className="glass-card rounded-[1.75rem] p-5 sm:p-6">
      <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-300">Inputs</p>

      <div className="mt-4 space-y-4">
        <label className="block text-sm text-slate-300">CPU cores: <span className="font-mono text-slate-100">{value.cpuCores.toFixed(1)}</span>
          <input type="range" min="0.5" max="32" step="0.5" value={value.cpuCores} onChange={(event) => onChange({ ...value, cpuCores: Number(event.target.value) })} className="mt-2 w-full accent-indigo-500" />
        </label>

        <label className="block text-sm text-slate-300">RAM: <span className="font-mono text-slate-100">{value.ramGb}GB</span>
          <input type="range" min="1" max="128" step="1" value={value.ramGb} onChange={(event) => onChange({ ...value, ramGb: Number(event.target.value) })} className="mt-2 w-full accent-cyan-400" />
        </label>

        <div className="grid gap-3 sm:grid-cols-2">
          <label className="block text-sm text-slate-300">Hours
            <input type="number" min="0" max="72" value={value.hours} onChange={(event) => onChange({ ...value, hours: Number(event.target.value) })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" />
          </label>
          <label className="block text-sm text-slate-300">Minutes
            <input type="number" min="0" max="59" value={value.minutes} onChange={(event) => onChange({ ...value, minutes: Number(event.target.value) })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" />
          </label>
        </div>

        <label className="block text-sm text-slate-300">Parallel jobs
          <input type="number" min="1" max="20" value={value.jobCount} onChange={(event) => onChange({ ...value, jobCount: Number(event.target.value) })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" />
        </label>

        <label className="flex items-center gap-2 text-sm text-slate-300">
          <input type="checkbox" checked={value.includeBaseFee} onChange={(event) => onChange({ ...value, includeBaseFee: event.target.checked })} />
          Include base fee per job
        </label>
      </div>
    </section>
  );
}
