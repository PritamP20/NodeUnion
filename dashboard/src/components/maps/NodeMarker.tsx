"use client";

import { Marker } from "react-simple-maps";
import type { MouseEvent, RefObject } from "react";
import { getStatusColor, type NodeHealthMapNode } from "@/lib/node-utils";

export function NodeMarker({
  node,
  containerRef,
  onHover,
  onLeave,
}: {
  node: NodeHealthMapNode;
  containerRef: RefObject<HTMLDivElement | null>;
  onHover: (payload: { node: NodeHealthMapNode; x: number; y: number }) => void;
  onLeave: () => void;
}) {
  const color = getStatusColor(node.status, node.knownRegion);

  function handleHover(event: MouseEvent<SVGGElement>) {
    const container = containerRef.current;
    if (!container) {
      return;
    }

    const markerRect = event.currentTarget.getBoundingClientRect();
    const containerRect = container.getBoundingClientRect();

    onHover({
      node,
      x: markerRect.left - containerRect.left + markerRect.width / 2,
      y: markerRect.top - containerRect.top,
    });
  }

  return (
    <Marker coordinates={node.coordinates}>
      <g
        role="presentation"
        style={{ cursor: "default" }}
        onMouseEnter={handleHover}
        onMouseMove={handleHover}
        onMouseLeave={onLeave}
      >
        <circle
          r={14}
          fill={color}
          fillOpacity={0.18}
          stroke={color}
          strokeOpacity={0.35}
          strokeWidth={1}
          style={{ animation: "ping 1.5s cubic-bezier(0, 0, 0.2, 1) infinite", transformBox: "fill-box", transformOrigin: "center" }}
        />
        <circle r={5} fill={color} stroke="rgba(255,255,255,0.85)" strokeWidth={0.75} />
      </g>
    </Marker>
  );
}