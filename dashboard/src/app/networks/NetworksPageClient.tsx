"use client";

import { useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import useSWR from "swr";
import { motion } from "framer-motion";
import {
  Activity,
  ArrowUpDown,
  Filter,
  Globe,
  Layers3,
  Server,
} from "lucide-react";
import { MapFilterBar, type MapStatusFilter } from "@/components/maps/MapFilterBar";
import { MapSummaryStrip } from "@/components/maps/MapSummaryStrip";
import { NodeHealthMap } from "@/components/maps/NodeHealthMap";
import { TopologyGraph } from "@/components/topology/TopologyGraph";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { LiveBadge } from "@/components/live-badge";
import { MetricCard } from "@/components/metric-card";
import { Sparkline } from "@/components/sparkline";
import { StatusDot } from "@/components/status-dot";
import { clampMin, fetchMainSnapshot, formatRelativeTime } from "@/lib/orchestrator-realtime";
import { toNodeHealthMapNode } from "@/lib/node-utils";
import { buildTopologyGraph } from "@/lib/topology-utils";

type Snapshot = Awaited<ReturnType<typeof fetchMainSnapshot>>;

type NetworkStatusFilter = "all" | "online" | "degraded" | "offline";
type NetworkSortMode = "activity" | "nodes";

type NodeRow = {
  nodeId: string;
  networkId: string;
  networkName: string;
  status: "online" | "degraded" | "offline";
  cpuPct: number;
  ramPct: number;
  lastHeartbeat: string;
  region: string;
  providerWallet: string;
};

type NetworkSummary = {
  networkId: string;
  name: string;
  description: string;
  status: "online" | "degraded" | "offline";
  totalNodes: number;
  onlineNodes: number;
  queueDepth: number;
};

function nodeStatusFromRaw(raw: string): "online" | "degraded" | "offline" {
  const lowerRaw = (raw ?? "").toLowerCase();
  if (lowerRaw === "online") return "online";
  if (lowerRaw === "degraded") return "degraded";
  return "offline";
}

function buildMapCounts(nodes: Array<{ status: "online" | "degraded" | "offline" }>) {
  return nodes.reduce(
    (counts, node) => {
      counts.all += 1;
      counts[node.status] += 1;
      return counts;
    },
    {
      all: 0,
      online: 0,
      degraded: 0,
      offline: 0,
    } satisfies Record<MapStatusFilter, number>,
  );
}

function buildSnapshotView(snapshot?: Snapshot) {
  const networks = snapshot?.networks ?? [];
  const nodes = snapshot?.nodes ?? [];
  const jobs = snapshot?.jobs ?? [];

  const summaries: NetworkSummary[] = networks.map((network) => {
    const networkNodes = nodes.filter((node) => node.network_id === network.network_id);
    const networkJobs = jobs.filter((job) => job.network_id === network.network_id);
    const onlineNodes = networkNodes.filter((node) => node.status !== "Offline").length;
    const queueDepth = networkJobs.filter((job) => ["Pending", "Scheduled", "Running"].includes(job.status)).length;
    const totalNodes = networkNodes.length;

    return {
      networkId: network.network_id,
      name: network.name,
      description: network.description || "Live compute network",
      status: nodeStatusFromRaw(network.status),
      totalNodes,
      onlineNodes,
      queueDepth,
    };
  });

  const nodeRows: NodeRow[] = nodes.map((node) => ({
    nodeId: node.node_id,
    networkId: node.network_id,
    networkName: networks.find((network) => network.network_id === node.network_id)?.name ?? node.network_id,
    status: nodeStatusFromRaw(node.status),
    cpuPct: Math.max(0, 100 - node.cpu_available_pct),
    ramPct: Math.max(0, 100 - Math.round((node.ram_available_mb / 64_000) * 100)),
    lastHeartbeat: formatRelativeTime(node.last_seen_epoch_secs),
    region: node.region || "unknown",
    providerWallet: node.provider_wallet || "unknown",
  }));

  const mapNodes = nodes.map((node) =>
    toNodeHealthMapNode({
      nodeId: node.node_id,
      networkId: node.network_id,
      networkName: networks.find((network) => network.network_id === node.network_id)?.name ?? node.network_id,
      status: node.status,
      region: node.region,
      lastSeenEpochSecs: node.last_seen_epoch_secs,
      cpuAvailablePct: node.cpu_available_pct,
      ramAvailableMb: node.ram_available_mb,
    }),
  );

  return { summaries, nodeRows, mapNodes };
}

export function NetworksPageClient() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const { data, error, isLoading, isValidating, mutate } = useSWR<Snapshot>("/api/main/snapshot", () => fetchMainSnapshot(), {
    refreshInterval: 30000,
    revalidateOnFocus: true,
  });

  const currentView =
    searchParams.get("view") === "list"
      ? "list"
      : searchParams.get("view") === "topology"
        ? "topology"
        : "map";

  const [selectedStatus, setSelectedStatus] = useState<NetworkStatusFilter>("all");
  const [selectedMapStatus, setSelectedMapStatus] = useState<MapStatusFilter>("all");
  const [sortMode, setSortMode] = useState<NetworkSortMode>("activity");
  const [selectedNetworkId, setSelectedNetworkId] = useState<string>("all");

  useEffect(() => {
    if (searchParams.get("view") === "map" || searchParams.get("view") === "list" || searchParams.get("view") === "topology") {
      return;
    }

    const params = new URLSearchParams(searchParams.toString());
    params.set("view", "map");
    router.replace(`${pathname}?${params.toString()}`, { scroll: false });
  }, [pathname, router, searchParams]);

  const lastUpdated = useMemo(() => (data ? new Date().toLocaleTimeString() : "waiting for snapshot"), [data]);

  const { summaries, nodeRows, mapNodes } = useMemo(() => buildSnapshotView(data), [data]);

  const mapCounts = useMemo(() => buildMapCounts(mapNodes), [mapNodes]);
  const topology = useMemo(() => buildTopologyGraph(data), [data]);
  const visibleMapNodes = useMemo(
    () => mapNodes.filter((node) => selectedMapStatus === "all" || node.status === selectedMapStatus),
    [mapNodes, selectedMapStatus],
  );

  const visibleSummaries = useMemo(() => {
    const filtered = summaries.filter((network) => selectedStatus === "all" || network.status === selectedStatus);

    return [...filtered].sort((left, right) => {
      if (sortMode === "activity") {
        const leftActivity = left.queueDepth + left.onlineNodes;
        const rightActivity = right.queueDepth + right.onlineNodes;
        return rightActivity - leftActivity;
      }

      return right.totalNodes - left.totalNodes;
    });
  }, [summaries, sortMode, selectedStatus]);

  const showLoadingSkeleton = isLoading && !data;

  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.3 }} className="space-y-6 pb-8">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <Globe size={24} className="text-indigo-400" />
            <h1 className="text-3xl font-bold text-slate-100">Networks</h1>
          </div>
          <p className="mt-1 text-sm text-slate-400">
            Live topology and metrics across all available networks
            <LiveBadge />
          </p>
        </div>
      </div>

      <Tabs value={currentView} onValueChange={(view) => router.push(`${pathname}?view=${view}`, { scroll: false })}>
        <TabsList>
          <TabsTrigger value="map" className="flex items-center gap-2">
            <Server size={14} />
            Map View
          </TabsTrigger>
          <TabsTrigger value="list" className="flex items-center gap-2">
            <Layers3 size={14} />
            List View
          </TabsTrigger>
          <TabsTrigger value="topology" className="flex items-center gap-2">
            <Globe size={14} />
            Topology View
          </TabsTrigger>
        </TabsList>

        <TabsContent value="map" className="space-y-6">
          <div className="grid gap-4 sm:grid-cols-2 md:grid-cols-3">
            <MetricCard label="Online Networks" value={String(summaries.filter((s) => s.status === "online").length)} icon={<Activity size={16} className="text-cyan-300" />} />
            <MetricCard
              label="Total Nodes"
              value={String(summaries.reduce((sum, s) => sum + s.totalNodes, 0))}
              icon={<Server size={16} className="text-indigo-300" />}
            />
            <MetricCard
              label="Jobs Queued"
              value={String(summaries.reduce((sum, s) => sum + s.queueDepth, 0))}
              icon={<ArrowUpDown size={16} className="text-cyan-300" />}
            />
          </div>

          <div className="flex items-center justify-between gap-2">
            <MapFilterBar counts={mapCounts} value={selectedMapStatus} onValueChange={setSelectedMapStatus} />
          </div>

          <NodeHealthMap
            nodes={visibleMapNodes}
            isLoading={showLoadingSkeleton}
            isValidating={isValidating}
            error={error as Error | undefined}
            onRetry={() => void mutate()}
          />
          <MapSummaryStrip
            total={mapCounts.all}
            online={mapCounts.online}
            degraded={mapCounts.degraded}
            offline={mapCounts.offline}
            loading={showLoadingSkeleton}
          />
        </TabsContent>

        <TabsContent value="list" className="space-y-6">
          <motion.section
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.05 }}
            className="rounded-xl border border-white/10 bg-gradient-to-br from-white/5 to-white/[0.02] p-6 backdrop-blur-sm"
          >
            <div className="mb-4 flex flex-col items-start justify-between gap-3 sm:flex-row sm:items-center">
              <div className="flex items-center gap-2">
                <Filter size={16} className="text-slate-400" />
                <span className="font-medium text-slate-200">Filter & Sort</span>
              </div>

              <div className="flex flex-wrap gap-2">
                <select
                  value={selectedStatus}
                  onChange={(e) => setSelectedStatus(e.target.value as NetworkStatusFilter)}
                  className="rounded-lg border border-white/10 bg-slate-900/50 px-3 py-2 text-xs text-slate-300"
                >
                  <option value="all">All Status</option>
                  <option value="online">Online</option>
                  <option value="degraded">Degraded</option>
                  <option value="offline">Offline</option>
                </select>

                <select
                  value={sortMode}
                  onChange={(e) => setSortMode(e.target.value as NetworkSortMode)}
                  className="rounded-lg border border-white/10 bg-slate-900/50 px-3 py-2 text-xs text-slate-300"
                >
                  <option value="activity">Sort: Activity</option>
                  <option value="nodes">Sort: Node Count</option>
                </select>
              </div>
            </div>

            <div className="space-y-2">
              {visibleSummaries.map((network) => (
                <div key={network.networkId} className="flex items-center justify-between rounded-lg border border-white/5 bg-white/[0.02] p-3 px-4 hover:bg-white/[0.05]">
                  <div className="flex min-w-0 flex-1 items-center gap-3">
                    <StatusDot tone={network.status} />
                    <div className="min-w-0 flex-1">
                      <p className="truncate font-medium text-slate-100">{network.name}</p>
                      <p className="truncate text-xs text-slate-500">{network.description}</p>
                    </div>
                  </div>

                  <div className="flex gap-4 text-right text-xs text-slate-400">
                    <span title="Online nodes">{network.onlineNodes} online</span>
                    <span title="Total nodes">{network.totalNodes} total</span>
                    <span title="Queued jobs">{network.queueDepth} jobs</span>
                  </div>
                </div>
              ))}
            </div>
          </motion.section>
        </TabsContent>

        <TabsContent value="topology" className="space-y-6">
          <TopologyGraph 
            nodes={topology.nodes} 
            edges={topology.edges}
            isLoading={showLoadingSkeleton}
            error={error as Error | undefined}
          />
        </TabsContent>
      </Tabs>
    </motion.div>
  );
}
