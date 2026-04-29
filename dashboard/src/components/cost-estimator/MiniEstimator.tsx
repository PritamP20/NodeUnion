"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { estimateCost, type EstimatorInput } from "@/lib/pricing";

export function MiniEstimator() {
  const [state, setState] = useState<EstimatorInput>({
    cpuCores: 2,
    ramGb: 8,
    hours: 2,
    minutes: 0,
    jobCount: 1,
    includeBaseFee: true,
  });

  const estimate = useMemo(() => estimateCost(state), [state]);

  return (
    <section className="glass-card rounded-[2rem] p-6 sm:p-8">
      <div className="flex items-end justify-between gap-3">
        <div>
          <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-300">Estimate Your Compute Cost</p>
          <h2 className="mt-2 text-2xl font-semibold text-slate-100">Quick budget preview</h2>
        </div>
        <Link href="/tools/cost-estimator" className="text-sm text-indigo-300 hover:text-indigo-200">Full estimator -&gt;</Link>
      </div>

      <div className="mt-5 grid gap-4 md:grid-cols-2">
        <div className="space-y-3">
          <label className="block text-sm text-slate-300">CPU: {state.cpuCores.toFixed(1)}
            <input type="range" min="0.5" max="32" step="0.5" value={state.cpuCores} onChange={(event) => setState({ ...state, cpuCores: Number(event.target.value) })} className="mt-2 w-full accent-indigo-500" />
          </label>
          <label className="block text-sm text-slate-300">RAM: {state.ramGb}GB
            <input type="range" min="1" max="128" step="1" value={state.ramGb} onChange={(event) => setState({ ...state, ramGb: Number(event.target.value) })} className="mt-2 w-full accent-cyan-400" />
          </label>
          <label className="block text-sm text-slate-300">Duration (hours)
            <input type="number" min="0" max="72" value={state.hours} onChange={(event) => setState({ ...state, hours: Number(event.target.value) })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" />
          </label>
        </div>

        <div className="rounded-xl border border-white/10 bg-[#0a0a0f] p-4">
          <p className="text-xs uppercase tracking-[0.22em] text-slate-400">Estimated total</p>
          <p className="mt-3 font-mono text-4xl text-indigo-300">{estimate.totalSol.toFixed(4)} SOL</p>
          <p className="mt-2 text-sm text-slate-400">${estimate.totalUsd.toFixed(2)} USD</p>
        </div>
      </div>
    </section>
  );
}
