"use client";

import type { NormalizedNodeStatus } from "@/lib/node-utils";

export type MapStatusFilter = Exclude<NormalizedNodeStatus, "unknown"> | "all";

const FILTER_LABELS: Array<{ value: MapStatusFilter; label: string }> = [
  { value: "all", label: "All" },
  { value: "online", label: "Online" },
  { value: "degraded", label: "Degraded" },
  { value: "offline", label: "Offline" },
];

export function MapFilterBar({
  counts,
  value,
  onValueChange,
}: {
  counts: Record<MapStatusFilter, number>;
  value: MapStatusFilter;
  onValueChange: (value: MapStatusFilter) => void;
}) {
  return (
    <div className="flex flex-wrap items-center gap-2">
      {FILTER_LABELS.map((item) => {
        const active = value === item.value;

        return (
          <button
            key={item.value}
            type="button"
            onClick={() => onValueChange(item.value)}
            className={`rounded-full border px-3 py-1 text-xs font-medium transition ${
              active
                ? "bg-indigo-500/20 border-indigo-500/50 text-indigo-300"
                : "bg-white/5 border-white/10 text-white/40"
            }`}
          >
            {item.label} <span className="font-mono">{counts[item.value]}</span>
          </button>
        );
      })}
    </div>
  );
}