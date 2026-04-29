"use client";

import { Home, Minus, Plus } from "lucide-react";

export function MapControls({
  onZoomIn,
  onZoomOut,
  onReset,
}: {
  onZoomIn: () => void;
  onZoomOut: () => void;
  onReset: () => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <button
        type="button"
        onClick={onZoomIn}
        className="rounded-md border border-white/10 bg-white/5 p-1.5 text-slate-200 hover:bg-white/10"
        aria-label="Zoom in"
      >
        <Plus size={14} />
      </button>
      <button
        type="button"
        onClick={onZoomOut}
        className="rounded-md border border-white/10 bg-white/5 p-1.5 text-slate-200 hover:bg-white/10"
        aria-label="Zoom out"
      >
        <Minus size={14} />
      </button>
      <button
        type="button"
        onClick={onReset}
        className="rounded-md border border-white/10 bg-white/5 p-1.5 text-slate-200 hover:bg-white/10"
        aria-label="Reset map view"
      >
        <Home size={14} />
      </button>
    </div>
  );
}