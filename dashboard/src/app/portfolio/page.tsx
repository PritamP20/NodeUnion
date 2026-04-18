"use client";

import { FormEvent, useEffect, useMemo, useState } from "react";
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  ArrowRight,
  BarChart3,
  CircleDollarSign,
  Clock3,
  Cpu,
  Rocket,
  Server,
  Wallet,
} from "lucide-react";
import {
  providerEarningsSeries,
  providerJobs,
  providerNodes,
  providerSummary,
  userJobs,
  userSpendingSeries,
  userSummary,
} from "@/lib/dashboard-data";
import {
  estimateJobCredits,
  fetchMainSnapshot,
  statusLabel,
  submitJob,
  type OrchestratorEntitlement,
  type OrchestratorJob,
  type OrchestratorNode,
  type OrchestratorSettlement,
} from "@/lib/orchestrator-realtime";

type PortfolioTab = "provider" | "user";

type RealTimeSnapshot = {
  nodes: OrchestratorNode[];
  jobs: OrchestratorJob[];
  entitlements: OrchestratorEntitlement[];
  settlements: OrchestratorSettlement[];
  usingMock: boolean;
};

function MiniStatCard({
  label,
  value,
  icon: Icon,
}: {
  label: string;
  value: string;
  icon: typeof Wallet;
}) {
  return (
    <div className="rounded-2xl border border-white/5 bg-white/5 p-4">
      <div className="flex items-center justify-between text-slate-400">
        <span className="text-[11px] uppercase tracking-[0.24em]">{label}</span>
        <Icon size={15} className="text-sky-300" />
      </div>
      <p className="mt-3 text-3xl font-semibold tracking-tight text-slate-100">{value}</p>
    </div>
  );
}

function statusTone(status: string) {
  switch (status) {
    case "Running":
      return "bg-emerald-500/15 text-emerald-300 border-emerald-500/20";
    case "Queued":
      return "bg-sky-500/15 text-sky-300 border-sky-500/20";
    case "Completed":
      return "bg-slate-500/15 text-slate-300 border-slate-500/20";
    default:
      return "bg-rose-500/15 text-rose-300 border-rose-500/20";
  }
}

function formatSol(value: number) {
  return `${value.toFixed(2)} SOL`;
}

function formatCredits(value: number) {
  return `${Math.max(0, Math.round(value)).toLocaleString()} credits`;
}

function mapNodeStatus(status: OrchestratorNode["status"]) {
  if (status === "Idle" || status === "Busy") return "Online";
  return status;
}

function jobDuration(createdAtEpoch: number) {
  const elapsedMins = Math.max(1, Math.round((Date.now() / 1000 - createdAtEpoch) / 60));
  if (elapsedMins < 60) return `${elapsedMins}m`;
  const hours = Math.floor(elapsedMins / 60);
  const mins = elapsedMins % 60;
  return `${hours}h ${mins}m`;
}

export default function PortfolioPage() {
  const defaultWallet = process.env.NEXT_PUBLIC_DEFAULT_USER_WALLET ?? "demo-user-wallet";
  const [tab, setTab] = useState<PortfolioTab>("provider");
  const [wallet, setWallet] = useState(defaultWallet);
  const [statusMessage, setStatusMessage] = useState("Ready to analyze earnings and spend.");
  const [refreshStamp, setRefreshStamp] = useState("mock data");
  const [snapshot, setSnapshot] = useState<RealTimeSnapshot>({
    nodes: [],
    jobs: [],
    entitlements: [],
    settlements: [],
    usingMock: true,
  });
  const [requestForm, setRequestForm] = useState({
    network: "college-a",
    image: "alpine:3.20",
    cpu: "0.25",
    ram: "128",
    command: "echo hello-nodeunion",
  });

  useEffect(() => {
    let cancelled = false;
    let timer: number | undefined;

    const refresh = async () => {
      let nextDelay = 12000;
      try {
        const snapshotData = await fetchMainSnapshot(wallet.trim() || undefined);
        const jobs = snapshotData.jobs;
        const nodes = snapshotData.nodes;
        const entitlements = snapshotData.entitlements;
        const settlements = snapshotData.settlements;

        if (cancelled) return;

        setSnapshot({
          jobs,
          nodes,
          entitlements,
          settlements,
          usingMock: !snapshotData.has_live_data,
        });
        setRefreshStamp(new Date().toLocaleTimeString());
      } catch {
        if (cancelled) return;
        setSnapshot((current) => ({
          ...current,
          usingMock: true,
        }));
        setRefreshStamp("mock fallback");
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
  }, [wallet]);

  const providerNodesLive = useMemo(() => {
    if (snapshot.usingMock || snapshot.nodes.length === 0) {
      return providerNodes;
    }

    return snapshot.nodes.map((node) => ({
      nodeId: node.node_id,
      networkId: node.network_id,
      region: node.region || "unknown",
      status: mapNodeStatus(node.status),
      uptime: node.status === "Offline" ? "stale" : "live",
      jobsCompleted: snapshot.jobs.filter((job) => job.assigned_node_id === node.node_id && job.status === "Done").length,
    }));
  }, [snapshot.jobs, snapshot.nodes, snapshot.usingMock]);

  const providerJobsLive = useMemo(() => {
    if (snapshot.usingMock || snapshot.jobs.length === 0) {
      return providerJobs;
    }

    return snapshot.jobs
      .filter((job) => job.status === "Done" || job.status === "Running" || job.status === "Scheduled")
      .sort((a, b) => b.created_at_epoch_secs - a.created_at_epoch_secs)
      .slice(0, 12)
      .map((job) => {
        const credits = estimateJobCredits(job);
        const solEstimate = credits / 100;
        return {
          jobId: job.job_id,
          duration: jobDuration(job.created_at_epoch_secs),
          cpuUsed: job.cpu_limit.toFixed(2),
          ramUsed: `${Math.round(job.ram_limit_mb / 1024)} GB`,
          solEarned: formatSol(solEstimate),
        };
      });
  }, [snapshot.jobs, snapshot.usingMock]);

  const providerEarningsSeriesLive = useMemo(() => {
    if (snapshot.usingMock || snapshot.jobs.length === 0) {
      return providerEarningsSeries;
    }

    const points = Array.from({ length: 30 }, (_, index) => {
      const date = new Date();
      date.setDate(date.getDate() - (29 - index));
      const dayKey = date.toISOString().slice(0, 10);
      const jobsForDay = snapshot.jobs.filter((job) => {
        const created = new Date(job.created_at_epoch_secs * 1000).toISOString().slice(0, 10);
        return created === dayKey && (job.status === "Done" || job.status === "Running");
      });
      const value = jobsForDay.reduce((sum, job) => sum + estimateJobCredits(job) / 100, 0);
      return {
        label: `${date.getMonth() + 1}/${date.getDate()}`,
        value: Number(value.toFixed(2)),
      };
    });

    return points;
  }, [snapshot.jobs, snapshot.usingMock]);

  const providerSummaryLive = useMemo(() => {
    if (snapshot.usingMock || snapshot.jobs.length === 0) {
      return providerSummary;
    }

    const completedJobs = snapshot.jobs.filter((job) => job.status === "Done");
    const runningJobs = snapshot.jobs.filter((job) => job.status === "Running" || job.status === "Scheduled");
    const totalCredits = completedJobs.reduce((sum, job) => sum + estimateJobCredits(job), 0);
    const monthCredits = completedJobs
      .filter((job) => {
        const date = new Date(job.created_at_epoch_secs * 1000);
        const now = new Date();
        return date.getFullYear() === now.getFullYear() && date.getMonth() === now.getMonth();
      })
      .reduce((sum, job) => sum + estimateJobCredits(job), 0);

    const onlineNodes = snapshot.nodes.filter((node) => node.status !== "Offline").length;
    const rejectionRate = snapshot.jobs.length > 0 ? ((snapshot.jobs.filter((job) => job.status === "Failed").length / snapshot.jobs.length) * 100).toFixed(1) : "0.0";

    return {
      totalEarned: formatSol(totalCredits / 100),
      earnedMonth: formatSol(monthCredits / 100),
      pendingSettlement: formatSol(runningJobs.reduce((sum, job) => sum + estimateJobCredits(job), 0) / 100),
      uptime: `${onlineNodes}/${snapshot.nodes.length || 1} online`,
      jobsCompleted: completedJobs.length,
      rejectionRate: `${rejectionRate}%`,
    };
  }, [snapshot.jobs, snapshot.nodes, snapshot.usingMock]);

  const userJobsLive = useMemo(() => {
    if (snapshot.usingMock || snapshot.jobs.length === 0) {
      return userJobs;
    }

    return snapshot.jobs
      .filter((job) => !wallet.trim() || (job.user_wallet ?? "") === wallet.trim())
      .sort((a, b) => b.created_at_epoch_secs - a.created_at_epoch_secs)
      .slice(0, 15)
      .map((job) => ({
        jobId: job.job_id,
        image: job.image,
        network: job.network_id,
        status: statusLabel(job.status) as "Running" | "Queued" | "Completed" | "Failed",
        cpuLimit: job.cpu_limit.toFixed(2),
        ramLimit: `${Math.round(job.ram_limit_mb / 1024)} GB`,
        duration: jobDuration(job.created_at_epoch_secs),
        credits: String(estimateJobCredits(job)),
      }));
  }, [snapshot.jobs, snapshot.usingMock, wallet]);

  const userSummaryLive = useMemo(() => {
    if (snapshot.usingMock) {
      return userSummary;
    }

    const entitlementBought = snapshot.entitlements.reduce((sum, item) => sum + item.bought_units, 0);
    const entitlementUsed = snapshot.entitlements.reduce((sum, item) => sum + item.used_units, 0);
    const monthSettlements = snapshot.settlements.filter((settlement) => {
      const date = new Date(settlement.created_at_epoch_secs * 1000);
      const now = new Date();
      return date.getMonth() === now.getMonth() && date.getFullYear() === now.getFullYear();
    });

    const totalSpendTokens = snapshot.settlements.reduce((sum, item) => sum + item.amount_tokens, 0);
    const monthSpendTokens = monthSettlements.reduce((sum, item) => sum + item.amount_tokens, 0);

    return {
      creditsRemaining: formatCredits(entitlementBought - entitlementUsed),
      spentMonth: formatSol(monthSpendTokens / 1_000_000_000),
      totalSpend: formatSol(totalSpendTokens / 1_000_000_000),
    };
  }, [snapshot.entitlements, snapshot.settlements, snapshot.usingMock]);

  const userSpendingSeriesLive = useMemo(() => {
    if (snapshot.usingMock || snapshot.settlements.length === 0) {
      return userSpendingSeries;
    }

    const monthKeys: string[] = [];
    const monthlyTotals: Record<string, number> = {};

    snapshot.settlements.forEach((settlement) => {
      const date = new Date(settlement.created_at_epoch_secs * 1000);
      const key = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
      if (!monthKeys.includes(key)) {
        monthKeys.push(key);
      }
      monthlyTotals[key] = (monthlyTotals[key] ?? 0) + settlement.amount_tokens;
    });

    return monthKeys
      .sort()
      .slice(-6)
      .map((key) => {
        const [year, month] = key.split("-");
        return {
          label: `${month}/${year.slice(2)}`,
          value: Number((monthlyTotals[key] / 1_000_000_000).toFixed(2)),
        };
      });
  }, [snapshot.settlements, snapshot.usingMock]);

  const providerView = (
    <section className="mt-6 space-y-6">
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MiniStatCard label="Total SOL earned" value={providerSummaryLive.totalEarned} icon={CircleDollarSign} />
        <MiniStatCard label="This month" value={providerSummaryLive.earnedMonth} icon={BarChart3} />
        <MiniStatCard label="Pending settlement" value={providerSummaryLive.pendingSettlement} icon={Clock3} />
        <MiniStatCard label="Uptime" value={providerSummaryLive.uptime} icon={Server} />
      </div>

      <div className="grid gap-6 xl:grid-cols-[1.1fr_0.9fr]">
        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <BarChart3 size={15} className="text-sky-400" />
            Daily earnings
          </div>
          <h2 className="mt-3 text-2xl font-semibold tracking-tight">SOL earned over the last 30 days</h2>
          <div className="mt-5 rounded-[1.35rem] border border-white/5 bg-[#0b1018] p-4">
            <ResponsiveContainer width="100%" height={320} minWidth={0}>
              <BarChart data={providerEarningsSeriesLive}>
                <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.06)" />
                <XAxis dataKey="label" tick={{ fill: "#8b949e", fontSize: 12 }} stroke="rgba(255,255,255,0.06)" />
                <YAxis tick={{ fill: "#8b949e", fontSize: 12 }} stroke="rgba(255,255,255,0.06)" />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#0d1117",
                    border: "1px solid rgba(255,255,255,0.08)",
                    borderRadius: 12,
                    color: "#e6edf3",
                  }}
                />
                <Bar dataKey="value" radius={[8, 8, 0, 0]}>
                  {providerEarningsSeriesLive.map((entry, index) => (
                    <Cell key={`${entry.label}-${index}`} fill={index % 3 === 0 ? "#3fb950" : "#2f81f7"} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        </article>

        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <Cpu size={15} className="text-sky-400" />
            Node health
          </div>
          <div className="mt-4 grid gap-3 sm:grid-cols-3">
            {[
              { label: "Uptime", value: providerSummaryLive.uptime },
              { label: "Jobs completed", value: String(providerSummaryLive.jobsCompleted) },
              { label: "Rejection rate", value: providerSummaryLive.rejectionRate },
            ].map((item) => (
              <div key={item.label} className="rounded-2xl border border-white/5 bg-white/5 p-4">
                <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">{item.label}</p>
                <p className="mt-2 text-2xl font-semibold text-slate-100">{item.value}</p>
              </div>
            ))}
          </div>

          <div className="mt-5 space-y-3">
            {providerNodesLive.map((node) => (
              <div key={node.nodeId} className="rounded-2xl border border-white/5 bg-white/5 p-4">
                <div className="flex items-center justify-between gap-3">
                  <div>
                    <p className="font-mono text-xs uppercase tracking-[0.28em] text-sky-300">{node.nodeId}</p>
                    <p className="mt-1 text-sm text-slate-300">
                      {node.networkId} · {node.region}
                    </p>
                  </div>
                  <span className="rounded-full border border-white/5 bg-black/20 px-3 py-1 text-xs text-slate-300">
                    {node.status}
                  </span>
                </div>
                <div className="mt-3 grid grid-cols-3 gap-3 text-sm">
                  <div className="rounded-xl bg-black/20 p-3">
                    <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Uptime</p>
                    <p className="mt-2 font-semibold text-slate-100">{node.uptime}</p>
                  </div>
                  <div className="rounded-xl bg-black/20 p-3">
                    <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Jobs</p>
                    <p className="mt-2 font-semibold text-slate-100">{node.jobsCompleted}</p>
                  </div>
                  <div className="rounded-xl bg-black/20 p-3">
                    <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Region</p>
                    <p className="mt-2 font-semibold text-slate-100">{node.region}</p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </article>
      </div>

      <div className="grid gap-6 xl:grid-cols-[1.2fr_0.8fr]">
        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <Wallet size={15} className="text-sky-400" />
            Completed jobs
          </div>
          <div className="mt-4 overflow-hidden rounded-[1.35rem] border border-white/5">
            <table className="min-w-full text-left text-sm">
              <thead className="bg-white/5 text-slate-400">
                <tr>
                  <th className="px-4 py-3 font-medium">Job ID</th>
                  <th className="px-4 py-3 font-medium">Duration</th>
                  <th className="px-4 py-3 font-medium">CPU used</th>
                  <th className="px-4 py-3 font-medium">RAM used</th>
                  <th className="px-4 py-3 font-medium">SOL earned</th>
                </tr>
              </thead>
              <tbody>
                {providerJobsLive.map((job) => (
                  <tr key={job.jobId} className="border-t border-white/5 text-slate-300">
                    <td className="px-4 py-4 font-mono text-xs text-slate-100">{job.jobId}</td>
                    <td className="px-4 py-4">{job.duration}</td>
                    <td className="px-4 py-4">{job.cpuUsed}</td>
                    <td className="px-4 py-4">{job.ramUsed}</td>
                    <td className="px-4 py-4 text-emerald-300">{job.solEarned}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </article>

        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <Rocket size={15} className="text-sky-400" />
            Provider summary
          </div>
          <p className="mt-4 text-sm leading-6 text-slate-300">
            Provider metrics are now derived from live jobs and nodes pulled from the orchestrator. If the backend is down, this panel falls back to the demo dataset.
          </p>
          <div className="mt-5 rounded-[1.35rem] border border-white/5 bg-white/5 p-4">
            <div className="grid gap-3 sm:grid-cols-2">
              <div className="rounded-2xl bg-black/20 p-4">
                <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Total earned</p>
                <p className="mt-2 text-2xl font-semibold text-slate-100">{providerSummaryLive.totalEarned}</p>
              </div>
              <div className="rounded-2xl bg-black/20 p-4">
                <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Pending</p>
                <p className="mt-2 text-2xl font-semibold text-slate-100">{providerSummaryLive.pendingSettlement}</p>
              </div>
            </div>
          </div>
        </article>
      </div>
    </section>
  );

  const userView = (
    <section className="mt-6 space-y-6">
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MiniStatCard label="Credits remaining" value={userSummaryLive.creditsRemaining} icon={Wallet} />
        <MiniStatCard label="Spent this month" value={userSummaryLive.spentMonth} icon={CircleDollarSign} />
        <MiniStatCard label="Total spend" value={userSummaryLive.totalSpend} icon={BarChart3} />
        <MiniStatCard label="Active jobs" value={String(userJobsLive.length)} icon={Rocket} />
      </div>

      <div className="grid gap-6 xl:grid-cols-[1.1fr_0.9fr]">
        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <BarChart3 size={15} className="text-sky-400" />
            Spending over time
          </div>
          <h2 className="mt-3 text-2xl font-semibold tracking-tight">Monthly compute spend</h2>
          <div className="mt-5 rounded-[1.35rem] border border-white/5 bg-[#0b1018] p-4">
            <ResponsiveContainer width="100%" height={320} minWidth={0}>
              <LineChart data={userSpendingSeriesLive}>
                <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.06)" />
                <XAxis dataKey="label" tick={{ fill: "#8b949e", fontSize: 12 }} stroke="rgba(255,255,255,0.06)" />
                <YAxis tick={{ fill: "#8b949e", fontSize: 12 }} stroke="rgba(255,255,255,0.06)" />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#0d1117",
                    border: "1px solid rgba(255,255,255,0.08)",
                    borderRadius: 12,
                    color: "#e6edf3",
                  }}
                />
                <Line type="monotone" dataKey="value" stroke="#2F81F7" strokeWidth={3} dot={false} />
                <Area type="monotone" dataKey="value" stroke="#3FB950" fillOpacity={0.16} fill="#3FB950" />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </article>

        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <Wallet size={15} className="text-sky-400" />
            Quick action
          </div>
          <h2 className="mt-3 text-2xl font-semibold">Submit new job</h2>
          <p className="mt-2 text-sm leading-6 text-slate-300">
            This form now submits directly to the orchestrator through the dashboard proxy.
          </p>

          <div className="mt-4 rounded-2xl border border-white/5 bg-black/20 px-4 py-3">
            <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">User wallet</p>
            <input
              value={wallet}
              onChange={(event) => setWallet(event.target.value)}
              className="mt-2 w-full rounded-xl border border-white/5 bg-black/40 px-3 py-2 text-sm outline-none transition focus:border-sky-400/50"
              placeholder="Enter user wallet"
            />
          </div>

          <form
            className="mt-5 grid gap-3"
            onSubmit={async (event: FormEvent) => {
              event.preventDefault();

              const cpu = Number.parseFloat(requestForm.cpu);
              const ram = Number.parseInt(requestForm.ram, 10);

              if (!wallet.trim() || !requestForm.network.trim() || !requestForm.image.trim() || !Number.isFinite(cpu) || !Number.isFinite(ram)) {
                setStatusMessage("Please provide wallet, network, image, cpu, and ram values.");
                return;
              }

              try {
                const response = await submitJob({
                  network_id: requestForm.network.trim(),
                  user_wallet: wallet.trim(),
                  image: requestForm.image.trim(),
                  cpu_limit: cpu,
                  ram_limit_mb: ram,
                  command: requestForm.command.trim() ? requestForm.command.trim().split(" ") : undefined,
                });
                setStatusMessage(`${response.message} (${response.job_id})`);
              } catch {
                setStatusMessage("Job submit failed. Check orchestrator availability and wallet entitlement.");
              }
            }}
          >
            <div className="grid gap-3 sm:grid-cols-2">
              <input
                value={requestForm.network}
                onChange={(event) => setRequestForm((current) => ({ ...current, network: event.target.value }))}
                className="rounded-2xl border border-white/5 bg-black/20 px-4 py-3 text-sm outline-none transition focus:border-sky-400/50"
                placeholder="Network"
              />
              <input
                value={requestForm.image}
                onChange={(event) => setRequestForm((current) => ({ ...current, image: event.target.value }))}
                className="rounded-2xl border border-white/5 bg-black/20 px-4 py-3 text-sm outline-none transition focus:border-sky-400/50"
                placeholder="Image"
              />
            </div>

            <div className="grid gap-3 sm:grid-cols-3">
              <input
                value={requestForm.cpu}
                onChange={(event) => setRequestForm((current) => ({ ...current, cpu: event.target.value }))}
                className="rounded-2xl border border-white/5 bg-black/20 px-4 py-3 text-sm outline-none transition focus:border-sky-400/50"
                placeholder="CPU"
              />
              <input
                value={requestForm.ram}
                onChange={(event) => setRequestForm((current) => ({ ...current, ram: event.target.value }))}
                className="rounded-2xl border border-white/5 bg-black/20 px-4 py-3 text-sm outline-none transition focus:border-sky-400/50"
                placeholder="RAM MB"
              />
              <input
                value={requestForm.command}
                onChange={(event) => setRequestForm((current) => ({ ...current, command: event.target.value }))}
                className="rounded-2xl border border-white/5 bg-black/20 px-4 py-3 text-sm outline-none transition focus:border-sky-400/50"
                placeholder="Command"
              />
            </div>

            <button className="mt-1 inline-flex items-center justify-center gap-2 rounded-full bg-sky-500 px-5 py-3 text-sm font-semibold text-white transition hover:bg-sky-400">
              Submit job <ArrowRight size={16} />
            </button>
          </form>

          <div className="mt-5 rounded-[1.35rem] border border-white/5 bg-white/5 p-4">
            <p className="text-sm uppercase tracking-[0.24em] text-slate-500">Status</p>
            <p className="mt-2 text-sm leading-6 text-slate-300">{statusMessage}</p>
          </div>
        </article>
      </div>

      <article className="glass-card rounded-[1.75rem] p-6">
        <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
          <Clock3 size={15} className="text-sky-400" />
          Job history
        </div>
        <div className="mt-4 overflow-hidden rounded-[1.35rem] border border-white/5">
          <table className="min-w-full text-left text-sm">
            <thead className="bg-white/5 text-slate-400">
              <tr>
                <th className="px-4 py-3 font-medium">Job ID</th>
                <th className="px-4 py-3 font-medium">Image</th>
                <th className="px-4 py-3 font-medium">Network</th>
                <th className="px-4 py-3 font-medium">Status</th>
                <th className="px-4 py-3 font-medium">CPU / RAM</th>
                <th className="px-4 py-3 font-medium">Duration</th>
                <th className="px-4 py-3 font-medium">Cost</th>
              </tr>
            </thead>
            <tbody>
              {userJobsLive.map((job) => (
                <tr key={job.jobId} className="border-t border-white/5 text-slate-300">
                  <td className="px-4 py-4 font-mono text-xs text-slate-100">{job.jobId}</td>
                  <td className="px-4 py-4">{job.image}</td>
                  <td className="px-4 py-4">{job.network}</td>
                  <td className="px-4 py-4">
                    <span className={`rounded-full border px-3 py-1 text-xs font-semibold ${statusTone(job.status)}`}>
                      {job.status}
                    </span>
                  </td>
                  <td className="px-4 py-4">
                    {job.cpuLimit} / {job.ramLimit}
                  </td>
                  <td className="px-4 py-4">{job.duration}</td>
                  <td className="px-4 py-4 text-emerald-300">{job.credits} credits</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </article>
    </section>
  );

  return (
    <main className="mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
      <section className="glass-card rounded-[2rem] p-6 sm:p-8">
        <div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
          <div>
            <p className="font-mono text-xs uppercase tracking-[0.3em] text-sky-300/90">Portfolio</p>
            <h1 className="mt-3 text-4xl font-semibold tracking-tight sm:text-5xl">
              Provider earnings and user spending, side by side.
            </h1>
            <p className="mt-3 max-w-3xl text-sm leading-6 text-slate-300 sm:text-base">
              Switch between provider and user views to inspect earnings, spend, node health, job history, and wallet-level billing telemetry.
            </p>
          </div>

          <div className="inline-flex rounded-full border border-white/5 bg-white/5 p-1 text-sm">
            {(["provider", "user"] as PortfolioTab[]).map((item) => (
              <button
                key={item}
                onClick={() => setTab(item)}
                className={`rounded-full px-4 py-2 font-semibold transition ${tab === item ? "bg-sky-500 text-white" : "text-slate-300 hover:text-slate-100"}`}
              >
                {item === "provider" ? "Provider View" : "User View"}
              </button>
            ))}
          </div>
        </div>

        <div className="mt-4 grid grid-cols-2 gap-3 md:grid-cols-4">
          <div className="rounded-2xl border border-white/5 bg-white/5 p-4">
            <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Feed</p>
            <p className={`mt-2 text-2xl font-semibold ${snapshot.usingMock ? "text-amber-300" : "text-emerald-300"}`}>
              {snapshot.usingMock ? "Mock" : "Live"}
            </p>
          </div>
          <div className="rounded-2xl border border-white/5 bg-white/5 p-4">
            <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Provider jobs</p>
            <p className="mt-2 text-2xl font-semibold text-slate-100">{providerJobsLive.length}</p>
          </div>
          <div className="rounded-2xl border border-white/5 bg-white/5 p-4">
            <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">User jobs</p>
            <p className="mt-2 text-2xl font-semibold text-slate-100">{userJobsLive.length}</p>
          </div>
          <div className="rounded-2xl border border-white/5 bg-white/5 p-4">
            <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">Last refresh</p>
            <p className="mt-2 text-2xl font-semibold text-slate-100">{refreshStamp}</p>
          </div>
        </div>
      </section>

      {tab === "provider" ? providerView : userView}
    </main>
  );
}
