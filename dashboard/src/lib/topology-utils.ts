import type { AggregatedSnapshot } from "@/lib/orchestrator-realtime";

export type GraphNode = {
  id: string;
  type: "network" | "node";
  label: string;
  status: string;
  nodeCount?: number;
  networkId?: string;
};

export type GraphEdge = {
  source: string;
  target: string;
  hasActiveJob: boolean;
};

const ACTIVE_JOB_STATES = new Set(["Pending", "Scheduled", "Running"]);

export function buildTopologyGraph(snapshot?: AggregatedSnapshot) {
  const networks = snapshot?.networks ?? [];
  const nodes = snapshot?.nodes ?? [];
  const jobs = snapshot?.jobs ?? [];

  const graphNodes: GraphNode[] = [];
  const graphEdges: GraphEdge[] = [];

  for (const network of networks) {
    const attachedNodes = nodes.filter((node) => node.network_id === network.network_id);

    graphNodes.push({
      id: `network:${network.network_id}`,
      type: "network",
      label: network.name,
      status: network.status,
      nodeCount: attachedNodes.length,
      networkId: network.network_id,
    });
  }

  for (const node of nodes) {
    const nodeId = `node:${node.node_id}`;
    const networkHub = `network:${node.network_id}`;

    graphNodes.push({
      id: nodeId,
      type: "node",
      label: node.node_id,
      status: node.status,
      networkId: node.network_id,
    });

    const hasActiveJob = jobs.some(
      (job) =>
        job.network_id === node.network_id &&
        job.assigned_node_id === node.node_id &&
        ACTIVE_JOB_STATES.has(job.status),
    );

    graphEdges.push({
      source: networkHub,
      target: nodeId,
      hasActiveJob,
    });
  }

  return { nodes: graphNodes, edges: graphEdges };
}
