import { CostBreakdownChart } from "@/components/cost-estimator/CostBreakdownChart";

type Props = {
  cpuCost: number;
  ramCost: number;
  baseFee: number;
  totalSol: number;
  totalUsd: number;
  hoursForOneSol: number;
};

function f(value: number) {
  return value.toFixed(4);
}

export function EstimatorResults({ cpuCost, ramCost, baseFee, totalSol, totalUsd, hoursForOneSol }: Props) {
  return (
    <section className="glass-card rounded-[1.75rem] p-5 sm:p-6">
      <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-300">Results</p>
      <p className="mt-3 font-mono text-5xl tracking-tight text-indigo-300">{f(totalSol)} SOL</p>
      <p className="mt-2 text-sm text-slate-400">${totalUsd.toFixed(2)} USD</p>

      <div className="mt-5 rounded-xl border border-white/10 bg-white/5 p-4 text-sm text-slate-300">
        <p className="flex items-center justify-between"><span>CPU cost</span><span className="font-mono text-slate-100">{f(cpuCost)} SOL</span></p>
        <p className="mt-2 flex items-center justify-between"><span>RAM cost</span><span className="font-mono text-slate-100">{f(ramCost)} SOL</span></p>
        <p className="mt-2 flex items-center justify-between"><span>Base fee</span><span className="font-mono text-slate-100">{f(baseFee)} SOL</span></p>
        <p className="mt-3 border-t border-white/10 pt-2 flex items-center justify-between"><span>Total</span><span className="font-mono text-indigo-300">{f(totalSol)} SOL</span></p>
      </div>

      <div className="mt-4">
        <CostBreakdownChart cpuCost={cpuCost} ramCost={ramCost} baseFee={baseFee} />
      </div>

      <p className="mt-4 text-sm text-slate-300">
        At this configuration, you can run for about <span className="font-mono text-slate-100">{hoursForOneSol.toFixed(2)} hours</span> per 1 SOL.
      </p>
    </section>
  );
}
