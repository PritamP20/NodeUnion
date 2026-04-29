"use client";

import { useMemo, useRef, useState } from "react";
import { AlertTriangle, Loader2 } from "lucide-react";
import { ComposableMap, Geographies, Geography, ZoomableGroup } from "react-simple-maps";
import { MapControls } from "@/components/maps/MapControls";
import { MapTooltip } from "@/components/maps/MapTooltip";
import { NodeMarker } from "@/components/maps/NodeMarker";
import type { NodeHealthMapNode } from "@/lib/node-utils";

const GEO_URL = "https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json";
const DEFAULT_CENTER: [number, number] = [0, 20];
const DEFAULT_ZOOM = 1;
const MIN_ZOOM = 0.85;
const MAX_ZOOM = 8;

type TooltipState = {
  node: NodeHealthMapNode;
  x: number;
  y: number;
} | null;

export function NodeHealthMap({
  nodes,
  isLoading,
  isValidating,
  error,
  onRetry,
}: {
  nodes: NodeHealthMapNode[];
  isLoading: boolean;
  isValidating: boolean;
  error: Error | undefined;
  onRetry: () => void;
}) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [center, setCenter] = useState<[number, number]>(DEFAULT_CENTER);
  const [zoom, setZoom] = useState(DEFAULT_ZOOM);
  const [tooltip, setTooltip] = useState<TooltipState>(null);

  const hasVisibleNodes = nodes.length > 0;
  const loadingFirstPass = isLoading && !hasVisibleNodes;

  const markers = useMemo(
    () =>
      nodes.map((node) => (
        <NodeMarker
          key={node.nodeId}
          node={node}
          containerRef={containerRef}
          onHover={setTooltip}
          onLeave={() => setTooltip(null)}
        />
      )),
    [nodes],
  );

  function updateZoom(nextZoom: number) {
    setZoom(Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, Number(nextZoom.toFixed(2)))));
  }

  function handleZoomIn() {
    updateZoom(zoom * 1.25);
  }

  function handleZoomOut() {
    updateZoom(zoom / 1.25);
  }

  function handleReset() {
    setCenter(DEFAULT_CENTER);
    setZoom(DEFAULT_ZOOM);
    setTooltip(null);
  }

  return (
    <div ref={containerRef} className="relative overflow-hidden rounded-[1.75rem] border border-white/10 bg-transparent">
      <div className="flex flex-wrap items-center justify-between gap-4 border-b border-white/10 px-4 py-4 sm:px-5">
        <div>
          <p className="text-[11px] uppercase tracking-[0.32em] text-cyan-300">Node health map</p>
          <h2 className="mt-2 text-2xl font-semibold tracking-tight text-slate-100">Live node placement by region</h2>
        </div>
        <div className="flex items-center gap-3">
          {isValidating ? (
            <span className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-slate-300">
              <Loader2 size={13} className="animate-spin text-emerald-300" />
              Syncing
            </span>
          ) : (
            <span className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-slate-300">
              <span className="h-2 w-2 animate-pulse rounded-full bg-emerald-400" />
              Live · updates every 30s
            </span>
          )}
        </div>
      </div>

      <div className="relative h-[500px] bg-transparent">
        <div className="absolute right-4 top-4 z-20">
          <MapControls onZoomIn={handleZoomIn} onZoomOut={handleZoomOut} onReset={handleReset} />
        </div>

        {loadingFirstPass ? (
          <div className="h-full rounded-[1.25rem] bg-white/5 animate-pulse" />
        ) : error ? (
          <div className="absolute inset-4 flex items-center justify-center rounded-[1.25rem] border border-rose-500/20 bg-[#0f0f1a]/95 p-6 text-sm text-slate-200 shadow-xl">
            <div className="flex max-w-xl items-start gap-3 rounded-2xl border border-white/10 bg-white/5 px-4 py-3">
              <AlertTriangle size={18} className="mt-0.5 shrink-0 text-rose-300" />
              <div className="min-w-0">
                <p className="font-medium text-slate-100">Could not load node data. Retrying in 30s.</p>
                <button
                  type="button"
                  onClick={onRetry}
                  className="mt-3 rounded-md border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-slate-100 hover:bg-white/10"
                >
                  Retry now
                </button>
              </div>
            </div>
          </div>
        ) : (
          <ComposableMap
            projection="geoMercator"
            projectionConfig={{ center: DEFAULT_CENTER, scale: 160 }}
            style={{ width: "100%", height: "100%" }}
          >
            <ZoomableGroup
              center={center}
              zoom={zoom}
              onMoveEnd={({ coordinates, zoom: nextZoom }: { coordinates: [number, number]; zoom: number }) => {
                setCenter(coordinates);
                updateZoom(nextZoom);
              }}
            >
              <Geographies geography={GEO_URL}>
                {({ geographies }) =>
                  geographies.map((geography) => (
                    <Geography
                      key={geography.rsmKey}
                      geography={geography}
                      style={{
                        default: { fill: "#1a1a2e", stroke: "#ffffff08", strokeWidth: 0.5, outline: "none" },
                        hover: { fill: "#1e1e3a", stroke: "#ffffff08", strokeWidth: 0.5, outline: "none" },
                        pressed: { fill: "#1a1a2e", stroke: "#ffffff08", strokeWidth: 0.5, outline: "none" },
                      }}
                    />
                  ))
                }
              </Geographies>
              {markers}
            </ZoomableGroup>
          </ComposableMap>
        )}

        {tooltip ? <MapTooltip node={tooltip.node} x={tooltip.x} y={tooltip.y} /> : null}

        {!loadingFirstPass && !error && nodes.length === 0 ? (
          <div className="pointer-events-none absolute inset-4 flex items-center justify-center rounded-[1.25rem] border border-white/10 bg-[#0f0f1a]/60 px-6 text-center text-sm text-slate-300">
            No nodes match the selected status filter.
          </div>
        ) : null}
      </div>
    </div>
  );
}