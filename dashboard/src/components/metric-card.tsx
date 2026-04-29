import type { ReactNode } from "react";

export function MetricCard({
  label,
  value,
  delta,
  icon,
}: {
  label: string;
  value: string;
  delta?: string;
  icon?: ReactNode;
}) {
  return (
    <article className="metric-card rounded-3xl p-5">
      <div className="flex items-center justify-between gap-3 text-slate-400">
        <span className="text-[11px] uppercase tracking-[0.28em]">{label}</span>
        {icon}
      </div>
      <p className="metric-value mt-4 text-3xl font-semibold text-slate-100">{value}</p>
      {delta ? <p className="mt-2 text-xs uppercase tracking-[0.22em] text-cyan-300">{delta}</p> : null}
    </article>
  );
}