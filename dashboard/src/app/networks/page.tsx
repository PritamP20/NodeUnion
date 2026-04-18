"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import {
  Activity,
  ChevronRight,
  Cpu,
  Globe,
  MapPinned,
  Network,
  Server,
} from "lucide-react";
import { networkCards, type NetworkCard } from "@/lib/dashboard-data";
import {
  clampMin,
  fetchMainSnapshot,
  formatRelativeTime,
} from "@/lib/orchestrator-realtime";

const REGION_COORDINATES: Record<string, { x: number; y: number }> = {
  "us-east-1": { x: 68, y: 36 },
  "us-west-2": { x: 28, y: 38 },
  "us-central-1": { x: 54, y: 47 },
  "eu-west-1": { x: 46, y: 28 },
  "eu-central-1": { x: 62, y: 30 },
  "ap-southeast-1": { x: 82, y: 62 },
  "ca-central-1": { x: 22, y: 30 },
  "sa-east-1": { x: 54, y: 72 },
};

function truncateWallet(wallet: string) {
  return wallet.length <= 10 ? wallet : `${wallet.slice(0, 4)}…${wallet.slice(-4)}`;
}

function Sparkline({ values }: { values: number[] }) {
  const max = Math.max(...values);
  const min = Math.min(...values);
  const range = Math.max(max - min, 1);
  const points = values
    .map((value, index) => {
      const x = (index / (values.length - 1)) * 100;
      const y = 100 - ((value - min) / range) * 100;
      return `${x},${y}`;
    })
    .join(" ");

  return (
    <svg viewBox="0 0 100 100" className="h-20 w-full">
      <defs>
        <linearGradient id="sparklineFill" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="rgba(47, 129, 247, 0.45)" />
          <stop offset="100%" stopColor="rgba(47, 129, 247, 0.02)" />
        </linearGradient>
      </defs>
      <polygon points={`0,100 ${points} 100,100`} fill="url(#sparklineFill)" opacity="0.55" />
      <polyline points={points} fill="none" stroke="#2F81F7" strokeWidth="4" strokeLinejoin="round" strokeLinecap="round" />
    </svg>
  );
}

function statusTone(status: string) {
  if (status === "Online" || status === "Idle" || status === "Busy" || status === "Scheduled") return "status-ok";
  if (status === "Draining") return "text-amber-300";
  return "status-bad";
}

function toTitleCase(text: string) {
  if (!text) return "unknown";
  return text
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function displayNetworkId(value: string) {
  const parts = value.split("::");
  return parts.length > 1 ? parts[1] : value;
}

export default function NetworksPage() {
  const [cards, setCards] = useState<NetworkCard[]>(networkCards);
  const [selectedNetworkId, setSelectedNetworkId] = useState(networkCards[0].networkId);
  const [isLive, setIsLive] = useState(false);
  const [lastRefresh, setLastRefresh] = useState<string>("mock data");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const sparklineHistory = useRef<Record<string, number[]>>({});

  useEffect(() => {
    let cancelled = false;
    let timer: number | undefined;

    const refresh = async () => {
      let nextDelay = 12000;
      try {
        const snapshot = await fetchMainSnapshot();
        const networks = snapshot.networks;
        const nodes = snapshot.nodes;
        const jobs = snapshot.jobs;

        if (cancelled) return;

        const byNetwork = networks.map((network) => {
          const sourceKey = network.orchestrator_url || "unknown-source";
          const uniqueNetworkId = `${sourceKey}::${network.network_id}`;
          const networkNodes = nodes.filter(
            (node) => node.network_id === network.network_id && (node.orchestrator_url || "unknown-source") === sourceKey,
          );
          const networkJobs = jobs.filter(
            (job) => job.network_id === network.network_id && (job.orchestrator_url || "unknown-source") === sourceKey,
          );
          const onlineNodes = networkNodes.filter((node) => node.status !== "Offline").length;
          const offlineNodes = networkNodes.length - onlineNodes;
          const activeJobs = networkJobs.filter((job) => ["Pending", "Scheduled", "Running"].includes(job.status)).length;
          const estimatedHours = networkJobs.reduce((sum, job) => {
            const elapsedHours = clampMin((Date.now() / 1000 - job.created_at_epoch_secs) / 3600, 0);
            return sum + Math.max(0.25, Math.min(elapsedHours, 8));
          }, 0);

          const previous = sparklineHistory.current[uniqueNetworkId] ?? [activeJobs];
          const nextSparkline = [...previous, activeJobs].slice(-10);
          sparklineHistory.current[uniqueNetworkId] = nextSparkline;

          const regionCounts = networkNodes.reduce<Record<string, number>>((acc, node) => {
            const region = node.region || "unknown";
            acc[region] = (acc[region] ?? 0) + 1;
            return acc;
          }, {});

          const dots = Object.entries(regionCounts).map(([region, online], index) => {
            const known = REGION_COORDINATES[region];
            const fallback = {
              x: 20 + (index * 17) % 60,
              y: 20 + (index * 13) % 60,
            };

            return {
              region,
              x: known?.x ?? fallback.x,
              y: known?.y ?? fallback.y,
              online,
            };
          });

          return {
            networkId: uniqueNetworkId,
            name: network.name,
            summary:
              network.description ||
              `${toTitleCase(network.status)} network from ${sourceKey.replace(/^https?:\/\//, "")}`,
            onlineNodes,
            offlineNodes,
            activeJobs,
            totalComputeHours: Math.round(estimatedHours),
            sparkline: nextSparkline,
            dots,
            nodes: networkNodes.map((node) => ({
              nodeId: node.node_id,
              region: node.region || "unknown",
              status:
                node.status === "Idle" || node.status === "Busy"
                  ? "Online"
                  : node.status === "Preempting"
                    ? "Draining"
                    : node.status,
              cpuUsagePct: Math.max(0, 100 - node.cpu_available_pct),
              ramUsageGb: Math.round(node.ram_available_mb / 1024),
              lastHeartbeat: formatRelativeTime(node.last_seen_epoch_secs),
              providerWallet: node.provider_wallet || "unknown",
            })),
          } satisfies NetworkCard;
        });

        if (byNetwork.length > 0) {
          setCards(byNetwork);
          setSelectedNetworkId((current) =>
            byNetwork.some((network) => network.networkId === current) ? current : byNetwork[0].networkId,
          );
        } else {
          setCards(networkCards);
        }

        setIsLive(snapshot.has_live_data);
        setErrorMessage(
          snapshot.sources.some((source) => !source.ok)
            ? `Some sources failed (${snapshot.sources.filter((source) => !source.ok).length}/${snapshot.sources.length})`
            : null,
        );
        setLastRefresh(new Date().toLocaleTimeString());
      } catch (error) {
        if (cancelled) return;
        setCards(networkCards);
        setIsLive(false);
        setErrorMessage(error instanceof Error ? error.message : "failed to fetch orchestrator snapshot");
        setLastRefresh("mock fallback");
        nextDelay = 45000;
      }

      if (!cancelled) {
        timer = window.setTimeout(() => {
          void refresh();
        }, nextDelay);
      }
    };

    void refresh();

    return () => {
      cancelled = true;
      if (timer) {
        window.clearTimeout(timer);
      }
    };
  }, []);

  const selectedNetwork = useMemo(
    () => cards.find((network) => network.networkId === selectedNetworkId) ?? cards[0],
    [cards, selectedNetworkId],
  );

  const aggregate = useMemo(() => {
    const totalNetworks = cards.length;
    const totalNodesOnline = cards.reduce((sum, network) => sum + network.onlineNodes, 0);
    const totalNodesOffline = cards.reduce((sum, network) => sum + network.offlineNodes, 0);
    const queueDepth = cards.reduce((sum, network) => sum + network.activeJobs, 0);

    return { totalNetworks, totalNodesOnline, totalNodesOffline, queueDepth };
  }, [cards]);

  if (!selectedNetwork) {
    return null;
  }

  return (
    <main className="mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
      <section className="glass-card rounded-[2rem] p-6 sm:p-8">
        <div className="flex flex-wrap items-center gap-3 text-[11px] uppercase tracking-[0.3em] text-slate-400">
          <span className="rounded-full border border-sky-500/20 bg-sky-500/10 px-3 py-1 text-sky-300">
            Network overview
          </span>
          <span>Live cluster topology</span>
        </div>

        <div className="mt-4 flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
          <div>
            <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">
              Active networks, node health, and compute flow.
            </h1>
            <p className="mt-3 max-w-3xl text-sm leading-6 text-slate-300 sm:text-base">
              Each card summarizes network health, sparkline activity, and current queue pressure.
              Select a network to inspect its node table and regional footprint.
            </p>
          </div>

          <div className="grid grid-cols-2 gap-3 md:grid-cols-3">
            {[
              { label: "Total networks", value: aggregate.totalNetworks, icon: Network },
              { label: "Nodes online", value: aggregate.totalNodesOnline, icon: Server },
              { label: "Queue depth", value: aggregate.queueDepth, icon: Activity },
            ].map((stat) => {
              const Icon = stat.icon;
              return (
                <div key={stat.label} className="rounded-2xl border border-white/5 bg-white/5 px-4 py-3">
                  <div className="flex items-center justify-between text-slate-400">
                    <span className="text-[11px] uppercase tracking-[0.22em]">{stat.label}</span>
                    <Icon size={15} className="text-sky-300" />
                  </div>
                  <p className="mt-2 text-2xl font-semibold">{stat.value}</p>
                </div>
              );
            })}
          </div>
        </div>
        <div className="mt-4 flex flex-wrap gap-3 text-xs uppercase tracking-[0.22em] text-slate-400">
          <span className={`rounded-full border px-3 py-1 ${isLive ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-300" : "border-amber-500/30 bg-amber-500/10 text-amber-300"}`}>
            {isLive ? "Live data" : "Mock fallback"}
          </span>
          <span className="rounded-full border border-white/5 bg-white/5 px-3 py-1">
            Last refresh {lastRefresh}
          </span>
          {errorMessage ? (
            <span className="rounded-full border border-rose-500/25 bg-rose-500/10 px-3 py-1 text-rose-300">
              {errorMessage.slice(0, 72)}
            </span>
          ) : null}
        </div>
      </section>

      <section className="mt-6 grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {cards.map((network, index) => {
          const selected = network.networkId === selectedNetworkId;
          return (
            <button
              key={network.networkId}
              onClick={() => setSelectedNetworkId(network.networkId)}
              className={`grid-fade rounded-[1.5rem] border p-5 text-left transition ${
                selected
                  ? "border-sky-400/40 bg-sky-500/10 shadow-lg shadow-sky-500/10"
                  : "border-white/5 bg-white/5 hover:border-sky-400/25 hover:bg-white/7"
              }`}
              style={{ animationDelay: `${index * 90}ms` }}
            >
              <div className="flex items-start justify-between gap-4">
                <div>
                  <p className="font-mono text-xs uppercase tracking-[0.28em] text-sky-300/90">
                    {displayNetworkId(network.networkId)}
                  </p>
                  <h2 className="mt-2 text-xl font-semibold text-slate-100">{network.name}</h2>
                  <p className="mt-2 text-sm leading-6 text-slate-300">{network.summary}</p>
                </div>
                <div className="rounded-full border border-white/5 bg-black/20 px-3 py-1 text-xs text-slate-300">
                  {selected ? "Selected" : <ChevronRight size={14} />}
                </div>
              </div>

              <div className="mt-4 grid grid-cols-2 gap-3 text-sm">
                <div className="rounded-2xl bg-black/20 p-3">
                  <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Nodes</p>
                  <p className="mt-1 text-lg font-semibold text-slate-100">
                    {network.onlineNodes}
                    <span className="text-slate-500"> / {network.offlineNodes} offline</span>
                  </p>
                </div>
                <div className="rounded-2xl bg-black/20 p-3">
                  <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Active jobs</p>
                  <p className="mt-1 text-lg font-semibold text-slate-100">{network.activeJobs}</p>
                </div>
                <div className="rounded-2xl bg-black/20 p-3">
                  <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Compute hours</p>
                  <p className="mt-1 text-lg font-semibold text-slate-100">{network.totalComputeHours.toLocaleString()}</p>
                </div>
                <div className="rounded-2xl bg-black/20 p-3">
                  <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Activity</p>
                  <Sparkline values={network.sparkline} />
                </div>
              </div>
            </button>
          );
        })}
      </section>

      <section className="mt-6 grid gap-6 xl:grid-cols-[0.95fr_1.05fr]">
        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <Globe size={15} className="text-sky-400" />
            Regional footprint
          </div>
          <h3 className="mt-3 text-2xl font-semibold">{selectedNetwork.name}</h3>
          <p className="mt-2 text-sm leading-6 text-slate-300">{selectedNetwork.summary}</p>

          <div className="mt-5 rounded-[1.5rem] border border-white/5 bg-[#0b1018] p-4">
            <div className="relative h-72 overflow-hidden rounded-[1.25rem] border border-white/5 bg-[radial-gradient(circle_at_50%_50%,rgba(47,129,247,0.15),transparent_40%),linear-gradient(180deg,rgba(22,27,34,0.85),rgba(13,17,23,0.95))]">
              <div className="absolute inset-x-6 top-1/2 h-px bg-white/5" />
              <div className="absolute inset-y-6 left-1/2 w-px bg-white/5" />
              {selectedNetwork.dots.map((dot) => (
                <div
                  key={dot.region}
                  className="absolute"
                  style={{ left: `${dot.x}%`, top: `${dot.y}%` }}
                >
                  <div className="h-4 w-4 rounded-full bg-emerald-400 shadow-[0_0_0_8px_rgba(63,185,80,0.15)]" />
                  <div className="mt-2 rounded-full border border-white/5 bg-black/75 px-3 py-1 text-[11px] text-slate-300">
                    {dot.region} · {dot.online} online
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="mt-4 flex flex-wrap gap-3 text-sm text-slate-300">
            <div className="rounded-full border border-white/5 bg-white/5 px-3 py-2">
              <span className="text-slate-500">Online</span> {selectedNetwork.onlineNodes}
            </div>
            <div className="rounded-full border border-white/5 bg-white/5 px-3 py-2">
              <span className="text-slate-500">Offline</span> {selectedNetwork.offlineNodes}
            </div>
            <div className="rounded-full border border-white/5 bg-white/5 px-3 py-2">
              <span className="text-slate-500">Jobs</span> {selectedNetwork.activeJobs}
            </div>
          </div>
        </article>

        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <Cpu size={15} className="text-sky-400" />
            Node detail view
          </div>

          <div className="mt-4 overflow-hidden rounded-[1.35rem] border border-white/5">
            <table className="min-w-full text-left text-sm">
              <thead className="bg-white/5 text-slate-400">
                <tr>
                  <th className="px-4 py-3 font-medium">Node ID</th>
                  <th className="px-4 py-3 font-medium">Region</th>
                  <th className="px-4 py-3 font-medium">Status</th>
                  <th className="px-4 py-3 font-medium">CPU / RAM</th>
                  <th className="px-4 py-3 font-medium">Last heartbeat</th>
                  <th className="px-4 py-3 font-medium">Provider wallet</th>
                </tr>
              </thead>
              <tbody>
                {selectedNetwork.nodes.map((node) => (
                  <tr key={node.nodeId} className="border-t border-white/5 text-slate-300">
                    <td className="px-4 py-4 font-mono text-xs text-slate-100">{node.nodeId}</td>
                    <td className="px-4 py-4">{node.region}</td>
                    <td className={`px-4 py-4 font-semibold ${statusTone(node.status)}`}>{node.status}</td>
                    <td className="px-4 py-4">{node.cpuUsagePct}% / {node.ramUsageGb} GB</td>
                    <td className="px-4 py-4">{node.lastHeartbeat}</td>
                    <td className="px-4 py-4 font-mono text-xs text-slate-400">{truncateWallet(node.providerWallet)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="mt-5 rounded-[1.35rem] border border-white/5 bg-white/5 p-4">
            <div className="flex items-center gap-2 text-sm uppercase tracking-[0.26em] text-slate-400">
              <MapPinned size={15} className="text-emerald-400" />
              Selected network metrics
            </div>
            <div className="mt-4 grid grid-cols-2 gap-3 text-sm sm:grid-cols-4">
              {[
                { label: "Active jobs", value: selectedNetwork.activeJobs },
                { label: "Online nodes", value: selectedNetwork.onlineNodes },
                { label: "Offline nodes", value: selectedNetwork.offlineNodes },
                { label: "Compute hours", value: selectedNetwork.totalComputeHours.toLocaleString() },
              ].map((item) => (
                <div key={item.label} className="rounded-2xl bg-black/20 p-3">
                  <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">{item.label}</p>
                  <p className="mt-2 text-lg font-semibold text-slate-100">{item.value}</p>
                </div>
              ))}
            </div>
          </div>
        </article>
      </section>
    </main>
  );
}