"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import {
  forceCenter,
  forceCollide,
  forceLink,
  forceManyBody,
  forceSimulation,
  type Simulation,
  type SimulationLinkDatum,
  type SimulationNodeDatum,
} from "d3-force";
import type { GraphEdge, GraphNode } from "@/lib/topology-utils";

type SimNode = SimulationNodeDatum & GraphNode;
type SimEdge = SimulationLinkDatum<SimNode> & GraphEdge;

export function useTopologySimulation({
  nodes,
  edges,
  width,
  height,
  autoLayout,
}: {
  nodes: GraphNode[];
  edges: GraphEdge[];
  width: number;
  height: number;
  autoLayout: boolean;
}) {
  const [renderNodes, setRenderNodes] = useState<SimNode[]>([]);
  const simulationRef = useRef<Simulation<SimNode, SimEdge> | null>(null);

  const seededNodes = useMemo(() => {
    return nodes.map((node, index) => ({
      ...node,
      x: width / 2 + Math.cos(index) * 120,
      y: height / 2 + Math.sin(index) * 120,
    })) as SimNode[];
  }, [nodes, width, height]);

  const seededEdges = useMemo(() => {
    return edges.map((edge) => ({ ...edge })) as SimEdge[];
  }, [edges]);

  useEffect(() => {
    if (seededNodes.length === 0) {
      setRenderNodes([]);
      simulationRef.current?.stop();
      simulationRef.current = null;
      return;
    }

    const simulation = forceSimulation<SimNode>(seededNodes)
      .force("link", forceLink<SimNode, SimEdge>(seededEdges).id((d) => d.id).distance(120))
      .force("charge", forceManyBody().strength(-300))
      .force("center", forceCenter(width / 2, height / 2))
      .force("collide", forceCollide<SimNode>().radius((node) => (node.type === "network" ? 34 : 16)));

    simulation.on("tick", () => {
      setRenderNodes([...seededNodes]);
    });

    if (!autoLayout) {
      simulation.stop();
    }

    simulationRef.current = simulation as Simulation<SimNode, SimEdge>;

    return () => {
      simulation.stop();
    };
  }, [seededNodes, seededEdges, width, height, autoLayout]);

  useEffect(() => {
    if (!simulationRef.current) return;

    if (autoLayout) {
      simulationRef.current.alpha(0.8).restart();
    } else {
      simulationRef.current.stop();
    }
  }, [autoLayout]);

  return {
    simulationRef,
    renderNodes,
  };
}
