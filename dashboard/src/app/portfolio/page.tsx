"use client";

import { useMemo, useState } from "react";
import useSWR from "swr";
import { motion } from "framer-motion";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  ArrowLeft,
  ArrowRight,
  CircleDollarSign,
  Clock3,
  Layers3,
  Server,
  Wallet,
} from "lucide-react";
import { JobStatusBadge } from "@/components/job-status-badge";
import { LiveBadge } from "@/components/live-badge";
import { MetricCard } from "@/components/metric-card";
import { StatusDot } from "@/components/status-dot";
import { estimateJobCredits, fetchMainSnapshot, statusLabel } from "@/lib/orchestrator-realtime";

type Snapshot = Awaited<ReturnType<typeof fetchMainSnapshot>>;

type ChartMode = "daily" | "weekly" | "monthly";
type JobStatusFilter = "all" | "running" | "queued" | "completed" | "failed";
type SortMode = "recent" | "oldest" | "credits";

function formatSol(value: number) {
  return `${value.toFixed(2)} SOL`;
}

function formatTokens(value: number) {
  return `${Math.max(0, Math.round(value)).toLocaleString()} credits`;
}

function formatDuration(createdAtEpoch: number) {
  const elapsedMins = Math.max(1, Math.round((Date.now() / 1000 - createdAtEpoch) / 60));
  if (elapsedMins < 60) return `${elapsedMins}m`;
  const hours = Math.floor(elapsedMins / 60);
  const mins = elapsedMins % 60;
  return `${hours}h ${mins}m`;
}

function nodeTone(status: string) {
  if (status === "Offline") return "offline";
  if (status === "Draining" || status === "Preempting") return "degraded";
  return "online";
}

function jobStatusTone(status: string) {
  if (status === "Running") return "online";
  if (status === "Queued") return "degraded";
  if (status === "Completed") return "online";
  return "offline";
}

function mapJobStatus(status: Snapshot["jobs"][number]["status"]) {
  return statusLabel(status) as "Running" | "Queued" | "Completed" | "Failed";
}

function buildChartSeries(settlements: Snapshot["settlements"], mode: ChartMode) {
  const now = new Date();

  if (mode === "daily") {
    return Array.from({ length: 30 }, (_, index) => {
      const date = new Date();
      date.setDate(now.getDate() - (29 - index));
      const dayKey = date.toISOString().slice(0, 10);
      const value = settlements
        .filter((settlement) => new Date(settlement.created_at_epoch_secs * 1000).toISOString().slice(0, 10) === dayKey)
        .reduce((sum, settlement) => sum + settlement.amount_tokens / 1_000_000_000, 0);

      return { label: `${date.getMonth() + 1}/${date.getDate()}`, value: Number(value.toFixed(2)) };
    });
  }

  if (mode === "weekly") {
    return Array.from({ length: 12 }, (_, index) => {
      const end = new Date();
      end.setDate(now.getDate() - index * 7);
      const start = new Date(end);
      start.setDate(end.getDate() - 6);

      const value = settlements
        .filter((settlement) => {
          const date = new Date(settlement.created_at_epoch_secs * 1000);
          return date >= start && date <= end;
        })
        .reduce((sum, settlement) => sum + settlement.amount_tokens / 1_000_000_000, 0);

      return {
        label: `W${12 - index}`,
        value: Number(value.toFixed(2)),
      };
    }).reverse();
  }

  return Array.from({ length: 12 }, (_, index) => {
    const date = new Date();
    date.setMonth(now.getMonth() - (11 - index));
    const monthKey = `${date.getFullYear()}-${date.getMonth()}`;
    const value = settlements
      .filter((settlement) => {
        const settlementDate = new Date(settlement.created_at_epoch_secs * 1000);
        return `${settlementDate.getFullYear()}-${settlementDate.getMonth()}` === monthKey;
      })
      .reduce((sum, settlement) => sum + settlement.amount_tokens / 1_000_000_000, 0);

    return {
      label: date.toLocaleString(undefined, { month: "short" }),
      value: Number(value.toFixed(2)),
    };
  });
}

export default function PortfolioPage() {
  const defaultWallet = process.env.NEXT_PUBLIC_DEFAULT_USER_WALLET ?? "";
  const [wallet, setWallet] = useState(defaultWallet);
  const [chartMode, setChartMode] = useState<ChartMode>("daily");
  const [jobFilter, setJobFilter] = useState<JobStatusFilter>("all");
  const [sortMode, setSortMode] = useState<SortMode>("recent");
  const [page, setPage] = useState(1);

  const walletQuery = wallet.trim();
  const { data, error, isLoading } = useSWR<Snapshot>(
    ["/api/main/snapshot", walletQuery],
    () => fetchMainSnapshot(walletQuery || undefined),
    { refreshInterval: 30000, revalidateOnFocus: true },
  );

  const lastUpdated = data ? new Date().toLocaleTimeString() : "waiting for snapshot";

  const walletScopedNodes = useMemo(() => {
    const nodes = data?.nodes ?? [];
    if (!walletQuery) return nodes;
    return nodes.filter((node) => node.provider_wallet === walletQuery);
  }, [data?.nodes, walletQuery]);

  const walletScopedJobs = useMemo(() => {
    const jobs = data?.jobs ?? [];
    if (!walletQuery) return jobs;
    return jobs.filter((job) => !job.user_wallet || job.user_wallet === walletQuery);
  }, [data?.jobs, walletQuery]);

  const walletScopedEntitlements = useMemo(() => {
    const entitlements = data?.entitlements ?? [];
    if (!walletQuery) return entitlements;
    return entitlements.filter((entitlement) => entitlement.user_wallet === walletQuery);
  }, [data?.entitlements, walletQuery]);

  const walletScopedSettlements = useMemo(() => {
    const settlements = data?.settlements ?? [];
    if (!walletQuery) return settlements;
    return settlements.filter(
      (settlement) => settlement.user_wallet === walletQuery || settlement.provider_wallet === walletQuery,
    );
  }, [data?.settlements, walletQuery]);

  const providerSummary = useMemo(() => {
    const completedJobs = walletScopedJobs.filter((job) => job.status === "Done");
    const activeNodes = walletScopedNodes.filter((node) => node.status !== "Offline").length;
    const earnedTokens = walletScopedSettlements.reduce((sum, settlement) => sum + settlement.amount_tokens, 0);

    return {
      totalEarned: formatSol(earnedTokens / 1_000_000_000),
      activeNodes,
      jobsCompleted: completedJobs.length,
    };
  }, [walletScopedJobs, walletScopedNodes, walletScopedSettlements]);

  const chartData = useMemo(() => buildChartSeries(walletScopedSettlements, chartMode), [chartMode, walletScopedSettlements]);

  const nodeCards = useMemo(() => {
    return walletScopedNodes.map((node) => {
      const jobsRun = walletScopedJobs.filter((job) => job.assigned_node_id === node.node_id && ["Done", "Running"].includes(job.status)).length;
      const uptimePct = node.status === "Offline" ? 0 : Math.min(99, 82 + Math.min(17, node.running_chunks * 3));

      return {
        nodeId: node.node_id,
        uptimePct,
        jobsRun,
        status: node.status,
        region: node.region || "unknown",
      };
    });
  }, [walletScopedJobs, walletScopedNodes]);

  const filteredJobs = useMemo(() => {
    const filtered = walletScopedJobs.filter((job) => {
      if (jobFilter === "all") return true;
      if (jobFilter === "running") return job.status === "Running";
      if (jobFilter === "queued") return job.status === "Pending" || job.status === "Scheduled";
      if (jobFilter === "completed") return job.status === "Done";
      return job.status === "Failed" || job.status === "Stopped" || job.status === "Preempted";
    });

    return [...filtered].sort((left, right) => {
      if (sortMode === "oldest") return left.created_at_epoch_secs - right.created_at_epoch_secs;
      if (sortMode === "credits") return estimateJobCredits(right) - estimateJobCredits(left);
      return right.created_at_epoch_secs - left.created_at_epoch_secs;
    });
  }, [jobFilter, sortMode, walletScopedJobs]);

  const pageSize = 8;
  const totalPages = Math.max(1, Math.ceil(filteredJobs.length / pageSize));
  const currentPage = Math.min(page, totalPages);
  const paginatedJobs = filteredJobs.slice((currentPage - 1) * pageSize, currentPage * pageSize);

  const activeEntitlements = walletScopedEntitlements;
  const settlements = walletScopedSettlements;

  return (
    <main className="mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
      <motion.section
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.35, ease: "easeOut" }}
        className="glass-card rounded-[2rem] p-6 sm:p-8"
      >
        <div className="flex flex-wrap items-center gap-3 text-[11px] uppercase tracking-[0.32em] text-slate-400">
          <LiveBadge />
          <span>Portfolio and earnings</span>
          <span>{error ? "snapshot error" : "wallet-scoped snapshot"}</span>
        </div>

        <div className="mt-4 flex flex-wrap items-end justify-between gap-4">
          <div>
            <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">Wallet-based earnings and usage overview.</h1>
            <p className="mt-3 max-w-3xl text-sm leading-7 text-slate-300 sm:text-base">
              Load a wallet to scope the dashboard to that provider or user view, then inspect earnings, node health,
              jobs, entitlements, and settlements.
            </p>
          </div>

          <div className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-slate-300">
            <p className="text-[11px] uppercase tracking-[0.28em] text-slate-500">Last updated</p>
            <p className="mt-2 font-mono text-sm text-slate-100">{lastUpdated}</p>
          </div>
        </div>

        <label className="mt-6 block text-sm text-slate-300">
          Wallet address
          <input
            value={wallet}
            onChange={(event) => {
              setWallet(event.target.value);
              setPage(1);
            }}
            className="mt-2 w-full rounded-2xl px-4 py-3 font-mono text-sm"
            placeholder="wallet address"
          />
        </label>

        <div className="mt-6 grid gap-3 sm:grid-cols-3">
          <MetricCard label="Total Earnings" value={isLoading ? "—" : providerSummary.totalEarned} icon={<CircleDollarSign size={16} className="text-cyan-300" />} />
          <MetricCard label="Active Nodes" value={isLoading ? "—" : providerSummary.activeNodes.toLocaleString()} icon={<Server size={16} className="text-indigo-300" />} />
          <MetricCard label="Jobs Completed" value={isLoading ? "—" : providerSummary.jobsCompleted.toLocaleString()} icon={<Layers3 size={16} className="text-cyan-300" />} />
        </div>
      </motion.section>

      <section className="mt-6 grid gap-6 xl:grid-cols-[0.96fr_1.04fr]">
        <motion.article
          initial={{ opacity: 0, y: 12 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, amount: 0.2 }}
          transition={{ duration: 0.35, ease: "easeOut" }}
          className="glass-card rounded-[2rem] p-6"
        >
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="text-[11px] uppercase tracking-[0.32em] text-cyan-300">Earnings chart</p>
              <h2 className="mt-2 text-2xl font-semibold tracking-tight text-slate-100">Daily, weekly, and monthly spend.</h2>
            </div>
            <div className="flex items-center gap-2 rounded-full border border-white/10 bg-white/5 p-1 text-xs uppercase tracking-[0.22em] text-slate-300">
              {(["daily", "weekly", "monthly"] as const).map((mode) => (
                <button
                  key={mode}
                  type="button"
                  onClick={() => setChartMode(mode)}
                  className={`rounded-full px-3 py-2 transition ${chartMode === mode ? "bg-indigo-500/20 text-slate-100" : "text-slate-400"}`}
                >
                  {mode}
                </button>
              ))}
            </div>
          </div>

          <div className="mt-5 h-72 rounded-2xl border border-white/10 bg-white/5 p-4">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData}>
                <defs>
                  <linearGradient id="earningsGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="#6366F1" stopOpacity={0.45} />
                    <stop offset="100%" stopColor="#22D3EE" stopOpacity={0.04} />
                  </linearGradient>
                </defs>
                <CartesianGrid stroke="rgba(255,255,255,0.08)" vertical={false} />
                <XAxis dataKey="label" stroke="#9aa3b2" tickLine={false} axisLine={false} />
                <YAxis stroke="#9aa3b2" tickLine={false} axisLine={false} tickFormatter={(value) => `${value} SOL`} />
                <Tooltip
                  contentStyle={{
                    background: "rgba(9,10,15,0.96)",
                    border: "1px solid rgba(255,255,255,0.12)",
                    borderRadius: 12,
                    color: "#f4f4f7",
                    fontFamily: "var(--font-mono)",
                  }}
                  cursor={{ stroke: "rgba(99,102,241,0.25)" }}
                />
                <Area type="monotone" dataKey="value" stroke="#6366F1" strokeWidth={2} fill="url(#earningsGradient)" />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </motion.article>

        <motion.article
          initial={{ opacity: 0, y: 12 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, amount: 0.2 }}
          transition={{ duration: 0.35, ease: "easeOut" }}
          className="glass-card rounded-[2rem] p-6"
        >
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <Server size={15} className="text-cyan-300" />
            Node summary
          </div>

          <div className="mt-5 grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {nodeCards.length === 0 ? (
              <div className="rounded-2xl border border-white/10 bg-white/5 p-4 text-sm text-slate-400 md:col-span-2 xl:col-span-3">
                No provider nodes match the loaded wallet yet.
              </div>
            ) : (
              nodeCards.map((node) => (
                <div key={node.nodeId} className="rounded-2xl border border-white/10 bg-white/5 p-4">
                  <div className="flex items-center justify-between gap-3">
                    <div>
                      <p className="font-mono text-xs text-slate-500">{node.nodeId}</p>
                      <p className="mt-1 text-sm text-slate-200">{node.region}</p>
                    </div>
                    <StatusDot tone={nodeTone(node.status)} />
                  </div>
                  <div className="mt-4 space-y-2 text-sm text-slate-300">
                    <p>Uptime: <span className="font-mono text-slate-100">{node.uptimePct}%</span></p>
                    <p>Jobs run: <span className="font-mono text-slate-100">{node.jobsRun}</span></p>
                    <p>Status: <span className="font-mono text-slate-100">{node.status}</span></p>
                  </div>
                </div>
              ))
            )}
          </div>
        </motion.article>
      </section>

      <section className="mt-6 grid gap-6 xl:grid-cols-[1.08fr_0.92fr]">
        <motion.article
          initial={{ opacity: 0, y: 12 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, amount: 0.2 }}
          transition={{ duration: 0.35, ease: "easeOut" }}
          className="glass-card rounded-[2rem] p-6"
        >
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="text-[11px] uppercase tracking-[0.32em] text-cyan-300">Job history</p>
              <h2 className="mt-2 text-2xl font-semibold tracking-tight text-slate-100">Paginated and sortable records.</h2>
            </div>
            <div className="flex flex-wrap items-center gap-2 text-xs uppercase tracking-[0.22em] text-slate-400">
              <select
                value={jobFilter}
                onChange={(event) => {
                  setJobFilter(event.target.value as JobStatusFilter);
                  setPage(1);
                }}
                className="rounded-full border border-white/10 bg-white/5 px-4 py-2 font-mono text-xs text-slate-100"
              >
                <option value="all">All</option>
                <option value="running">Running</option>
                <option value="queued">Queued</option>
                <option value="completed">Completed</option>
                <option value="failed">Failed</option>
              </select>
              <select
                value={sortMode}
                onChange={(event) => {
                  setSortMode(event.target.value as SortMode);
                  setPage(1);
                }}
                className="rounded-full border border-white/10 bg-white/5 px-4 py-2 font-mono text-xs text-slate-100"
              >
                <option value="recent">Recent</option>
                <option value="oldest">Oldest</option>
                <option value="credits">Credits</option>
              </select>
            </div>
          </div>

          <div className="mt-5 overflow-hidden rounded-2xl border border-white/10">
            <table className="min-w-full text-left text-sm">
              <thead className="bg-white/5 text-slate-400">
                <tr>
                  <th className="px-4 py-3 font-medium">Job</th>
                  <th className="px-4 py-3 font-medium">Image</th>
                  <th className="px-4 py-3 font-medium">Status</th>
                  <th className="px-4 py-3 font-medium">Network</th>
                  <th className="px-4 py-3 font-medium">Duration</th>
                  <th className="px-4 py-3 font-medium">Credits</th>
                </tr>
              </thead>
              <tbody>
                {paginatedJobs.length === 0 ? (
                  <tr>
                    <td className="px-4 py-6 text-slate-400" colSpan={6}>
                      No jobs match the current filters.
                    </td>
                  </tr>
                ) : (
                  paginatedJobs.map((job) => (
                    <tr key={job.job_id} className="border-t border-white/10 text-slate-300">
                      <td className="px-4 py-4 font-mono text-xs text-slate-100">{job.job_id}</td>
                      <td className="px-4 py-4 text-slate-300">{job.image}</td>
                      <td className="px-4 py-4">
                        <span className="inline-flex items-center gap-2">
                          <StatusDot tone={jobStatusTone(mapJobStatus(job.status))} />
                          <JobStatusBadge state={mapJobStatus(job.status)} />
                        </span>
                      </td>
                      <td className="px-4 py-4 text-slate-300">{job.network_id}</td>
                      <td className="px-4 py-4 text-slate-300">{formatDuration(job.created_at_epoch_secs)}</td>
                      <td className="px-4 py-4 font-mono text-slate-100">{estimateJobCredits(job)}</td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>

          <div className="mt-4 flex items-center justify-between gap-3">
            <button
              type="button"
              onClick={() => setPage((current) => Math.max(1, current - 1))}
              disabled={currentPage === 1}
              className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-100 disabled:opacity-50"
            >
              <ArrowLeft size={15} /> Previous
            </button>
            <p className="font-mono text-xs uppercase tracking-[0.24em] text-slate-500">
              Page {currentPage} of {totalPages}
            </p>
            <button
              type="button"
              onClick={() => setPage((current) => Math.min(totalPages, current + 1))}
              disabled={currentPage === totalPages}
              className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-100 disabled:opacity-50"
            >
              Next <ArrowRight size={15} />
            </button>
          </div>
        </motion.article>

        <div className="space-y-6">
          <motion.article
            initial={{ opacity: 0, y: 12 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, amount: 0.2 }}
            transition={{ duration: 0.35, ease: "easeOut" }}
            className="glass-card rounded-[2rem] p-6"
          >
            <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
              <Wallet size={15} className="text-cyan-300" />
              Entitlements
            </div>

            <div className="mt-5 space-y-3">
              {activeEntitlements.length === 0 ? (
                <p className="text-sm text-slate-400">No entitlements available for the loaded wallet.</p>
              ) : (
                activeEntitlements.map((entitlement) => {
                  const remaining = entitlement.bought_units - entitlement.used_units;
                  const active = remaining > 0;

                  return (
                    <div key={entitlement.entitlement_id} className="rounded-2xl border border-white/10 bg-white/5 p-4">
                      <div className="flex items-center justify-between gap-3">
                        <div>
                          <p className="font-mono text-xs text-slate-500">{entitlement.entitlement_id}</p>
                          <p className="mt-1 text-sm text-slate-100">{entitlement.network_id}</p>
                        </div>
                        <StatusDot tone={active ? "online" : "offline"} />
                      </div>

                      <div className="mt-3 grid gap-2 text-sm text-slate-300 sm:grid-cols-2">
                        <p>Remaining: <span className="font-mono text-slate-100">{formatTokens(remaining)}</span></p>
                        <p>Expiry: <span className="font-mono text-slate-100">{active ? "active" : "expired"}</span></p>
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          </motion.article>

          <motion.article
            initial={{ opacity: 0, y: 12 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, amount: 0.2 }}
            transition={{ duration: 0.35, ease: "easeOut" }}
            className="glass-card rounded-[2rem] p-6"
          >
            <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
              <CircleDollarSign size={15} className="text-indigo-300" />
              Settlements
            </div>

            <div className="mt-5 space-y-3">
              {settlements.length === 0 ? (
                <p className="text-sm text-slate-400">No settlements available for the loaded wallet.</p>
              ) : (
                settlements.map((settlement) => (
                  <div key={settlement.settlement_id} className="rounded-2xl border border-white/10 bg-white/5 p-4">
                    <div className="flex items-center justify-between gap-3">
                      <div>
                        <p className="font-mono text-xs text-slate-500">{settlement.settlement_id}</p>
                        <p className="mt-1 text-sm text-slate-100">{settlement.network_id}</p>
                      </div>
                      <p className="font-mono text-sm text-slate-100">{formatSol(settlement.amount_tokens / 1_000_000_000)}</p>
                    </div>

                    <div className="mt-3 flex flex-wrap items-center justify-between gap-3 text-sm text-slate-300">
                      <p>{new Date(settlement.created_at_epoch_secs * 1000).toLocaleString()}</p>
                      <p className="text-slate-500">Tx status: {settlement.tx_status || "pending"}</p>
                    </div>
                  </div>
                ))
              )}
            </div>
          </motion.article>
        </div>
      </section>

      <section className="mt-6 rounded-[2rem] border border-white/10 bg-white/5 p-6 text-sm leading-6 text-slate-300">
        <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
          <Clock3 size={15} className="text-cyan-300" />
          Notes
        </div>
        <p className="mt-4">
          The portfolio view scopes to the wallet you enter above, then pulls jobs, entitlements, and settlements from the live orchestrator snapshot. The chart and tables refresh automatically so earnings and usage stay current.
        </p>
      </section>
    </main>
  );
}