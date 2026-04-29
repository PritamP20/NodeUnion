"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { select } from "d3-selection";
import { drag } from "d3-drag";
import { zoom, zoomIdentity, type D3ZoomEvent } from "d3-zoom";
import { Loader2 } from "lucide-react";
import { MapTooltip } from "@/components/maps/MapTooltip";
import { TopologyControls } from "@/components/topology/TopologyControls";
import { TopologyLegend } from "@/components/topology/TopologyLegend";
import { useTopologySimulation } from "@/components/topology/useTopologySimulation";
import { getStatusColor, normalizeNodeStatus, type NodeHealthMapNode } from "@/lib/node-utils";
import type { GraphEdge, GraphNode } from "@/lib/topology-utils";

const WIDTH = 1120;
const HEIGHT = 600;

type TooltipState = {
  node: NodeHealthMapNode;
  x: number;
  y: number;
} | null;

function mapNodeToTooltip(node: GraphNode): NodeHealthMapNode {
  return {
    nodeId: node.label,
    networkId: node.networkId ?? node.id,
    networkName: node.type === "network" ? node.label : node.networkId ?? "network",
    status: normalizeNodeStatus(node.status),
    region: "unknown",
    coordinates: [0, 0],
    knownRegion: true,
    lastSeenEpochSecs: Math.floor(Date.now() / 1000),
  };
}

function getNodeRadius(node: GraphNode) {
  return node.type === "network" ? 28 : 12;
}

export function TopologyGraph({
  nodes,
  edges,
  isLoading,
  error,
}: {
  nodes: GraphNode[];
  edges: GraphEdge[];
  isLoading: boolean;
  error?: Error;
}) {
  const [showJobFlow, setShowJobFlow] = useState(true);
  const [autoLayout, setAutoLayout] = useState(true);
  const [transform, setTransform] = useState(zoomIdentity);
  const [tooltip, setTooltip] = useState<TooltipState>(null);

  const svgRef = useRef<SVGSVGElement | null>(null);
  const overlayRef = useRef<HTMLDivElement | null>(null);

  const { simulationRef, renderNodes } = useTopologySimulation({
    nodes,
    edges,
    width: WIDTH,
    height: HEIGHT,
    autoLayout,
  });

  const edgeCoordinates = useMemo(() => {
    const index = new Map(renderNodes.map((node) => [node.id, node]));

    return edges
      .map((edge) => {
        const source = index.get(edge.source);
        const target = index.get(edge.target);
        if (!source || !target) return undefined;
        return { edge, source, target };
      })
      .filter(Boolean) as Array<{ edge: GraphEdge; source: GraphNode & { x: number; y: number }; target: GraphNode & { x: number; y: number } }>;
  }, [edges, renderNodes]);

  useEffect(() => {
    if (!svgRef.current) return;

    const svg = select(svgRef.current);
    const zoomBehavior = zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.5, 2.5])
      .on("zoom", (event: D3ZoomEvent<SVGSVGElement, unknown>) => {
        setTransform(event.transform);
      });

    svg.call(zoomBehavior);

    return () => {
      svg.on(".zoom", null);
    };
  }, []);

  useEffect(() => {
    if (!svgRef.current || !simulationRef.current) {
      return;
    }

    const svg = select(svgRef.current);

    const dragBehavior = drag<SVGCircleElement, GraphNode & { x?: number; y?: number; fx?: number | null; fy?: number | null }>()
      .on("start", (event, datum) => {
        if (!event.active) {
          simulationRef.current?.alphaTarget(0.25).restart();
        }
        datum.fx = datum.x;
        datum.fy = datum.y;
      })
      .on("drag", (event, datum) => {
        datum.fx = event.x;
        datum.fy = event.y;
      })
      .on("end", (event, datum) => {
        if (!event.active) {
          simulationRef.current?.alphaTarget(0);
        }
        if (autoLayout) {
          datum.fx = null;
          datum.fy = null;
        }
      });

    svg.selectAll<SVGCircleElement, GraphNode>("circle[data-topology-node='true']").call(dragBehavior);
  }, [autoLayout, renderNodes, simulationRef]);

  const resetView = () => {
    setTransform(zoomIdentity);
  };

  const onNodeHover = (event: React.MouseEvent, node: GraphNode) => {
    const container = overlayRef.current;
    if (!container) return;

    const containerRect = container.getBoundingClientRect();

    setTooltip({
      node: mapNodeToTooltip(node),
      x: event.clientX - containerRect.left,
      y: event.clientY - containerRect.top,
    });
  };

  if (isLoading) {
    return (
      <div className="glass-card flex h-[600px] items-center justify-center rounded-[1.75rem] text-slate-300">
        <Loader2 size={18} className="animate-spin" />
        <span className="ml-2">Building topology graph...</span>
      </div>
    );
  }

  if (error) {
    return <div className="rounded-2xl border border-red-500/20 bg-red-500/10 p-5 text-sm text-red-200">Failed to load topology graph.</div>;
  }

  return (
    <div ref={overlayRef} className="relative overflow-hidden rounded-[1.75rem] border border-white/10 bg-[#0a0a0f]">
      <div className="absolute right-4 top-4 z-20">
        <TopologyControls
          showJobFlow={showJobFlow}
          autoLayout={autoLayout}
          onToggleJobFlow={() => setShowJobFlow((value) => !value)}
          onToggleAutoLayout={() => setAutoLayout((value) => !value)}
          onResetView={resetView}
        />
      </div>

      <div className="absolute bottom-4 left-4 z-20">
        <TopologyLegend />
      </div>

      <svg ref={svgRef} viewBox={`0 0 ${WIDTH} ${HEIGHT}`} className="h-[600px] w-full">
        <g transform={transform.toString()}>
          {edgeCoordinates.map(({ edge, source, target }, index) => (
            <line
              key={`${edge.source}-${edge.target}-${index}`}
              x1={source.x ?? 0}
              y1={source.y ?? 0}
              x2={target.x ?? 0}
              y2={target.y ?? 0}
              stroke={edge.hasActiveJob && showJobFlow ? "#6366f1" : "#ffffff15"}
              strokeWidth={edge.hasActiveJob && showJobFlow ? 1.6 : 1}
              strokeDasharray={edge.hasActiveJob && showJobFlow ? "6 4" : undefined}
              style={
                edge.hasActiveJob && showJobFlow
                  ? ({ animation: "topology-flow 1.2s linear infinite" } as React.CSSProperties)
                  : undefined
              }
            />
          ))}

          {renderNodes.map((node) => {
            const radius = getNodeRadius(node);
            const fill = node.type === "network" ? "#6366f1" : getStatusColor(normalizeNodeStatus(node.status));

            return (
              <g key={node.id} transform={`translate(${node.x ?? 0}, ${node.y ?? 0})`}>
                <circle
                  data-topology-node="true"
                  r={radius}
                  fill={fill}
                  stroke="rgba(255,255,255,0.75)"
                  strokeWidth={node.type === "network" ? 1.1 : 0.8}
                  onMouseEnter={(event) => onNodeHover(event, node)}
                  onMouseMove={(event) => onNodeHover(event, node)}
                  onMouseLeave={() => setTooltip(null)}
                />
                <text
                  y={radius + 14}
                  textAnchor="middle"
                  className="fill-slate-200"
                  style={{ fontSize: node.type === "network" ? 12 : 10, fontFamily: "var(--font-mono)" }}
                >
                  {node.type === "network" ? node.label : node.label.slice(0, 10)}
                </text>
              </g>
            );
          })}
        </g>
      </svg>

      {tooltip ? <MapTooltip node={tooltip.node} x={tooltip.x} y={tooltip.y} /> : null}
    </div>
  );
}
