import { MetricCard } from "@/components/metric-card";

function SummaryDot({ className }: { className: string }) {
  return <span aria-hidden="true" className={`inline-flex h-2.5 w-2.5 rounded-full ${className}`} />;
}

export function MapSummaryStrip({
  total,
  online,
  degraded,
  offline,
  loading = false,
}: {
  total: number;
  online: number;
  degraded: number;
  offline: number;
  loading?: boolean;
}) {
  if (loading) {
    return (
      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        {Array.from({ length: 4 }, (_, index) => (
          <div key={index} className="metric-card rounded-3xl p-5">
            <div className="h-3 w-24 rounded-full bg-white/10" />
            <div className="mt-4 h-10 w-20 rounded-xl bg-white/10" />
            <div className="mt-3 h-3 w-28 rounded-full bg-white/10" />
          </div>
        ))}
      </div>
    );
  }

  return (
    <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
      <MetricCard label="Total Nodes" value={total.toLocaleString()} icon={<SummaryDot className="bg-slate-300/80" />} />
      <MetricCard label="Online" value={online.toLocaleString()} icon={<SummaryDot className="bg-emerald-400" />} />
      <MetricCard label="Degraded" value={degraded.toLocaleString()} icon={<SummaryDot className="bg-amber-400" />} />
      <MetricCard label="Offline" value={offline.toLocaleString()} icon={<SummaryDot className="bg-rose-400" />} />
    </div>
  );
}