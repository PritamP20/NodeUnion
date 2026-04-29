import { StatusDot } from "@/components/status-dot";

export function LiveBadge({ label = "LIVE" }: { label?: string }) {
  return (
    <span className="live-badge rounded-full px-3 py-1.5 text-[11px] font-semibold uppercase tracking-[0.24em]">
      <StatusDot tone="online" />
      {label}
    </span>
  );
}