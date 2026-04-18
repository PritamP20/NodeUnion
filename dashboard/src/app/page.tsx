"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { Job, Network, Node, fetchOverview } from "@/lib/orchestrator";

export default function LandingPage() {
  const [networks, setNetworks] = useState<Network[]>([]);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [jobs, setJobs] = useState<Job[]>([]);
  const [statusMessage, setStatusMessage] = useState(
    "Click refresh to load live overview from the orchestrator",
  );

  async function refreshOverview() {
    try {
      const data = await fetchOverview();
      setNetworks(data.networks);
      setNodes(data.nodes);
      setJobs(data.jobs);
      setStatusMessage("Live overview synced from orchestrator");
    } catch (error) {
      setStatusMessage(`Unable to load live overview: ${(error as Error).message}`);
    }
  }

  const metrics = useMemo(() => {
    const idleNodes = nodes.filter((node) => node.is_idle).length;
    const busyNodes = nodes.length - idleNodes;
    const runningJobs = jobs.filter(
      (job) => job.status === "Running" || job.status === "Scheduled",
    ).length;

    return {
      networks: networks.length,
      providers: nodes.length,
      idleNodes,
      busyNodes,
      jobs: jobs.length,
      runningJobs,
    };
  }, [jobs, networks, nodes]);

  return (
    <main className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <section className="glass-card rounded-3xl p-7 sm:p-9">
        <p className="font-mono text-xs tracking-[0.2em] text-cyan-200/70">
          NODEUNION / BLOCKCHAIN COMPUTE EXCHANGE
        </p>
        <h1 className="mt-2 text-3xl font-semibold tracking-tight sm:text-5xl">
          A complete control plane for decentralized compute provisioning
        </h1>
        <p className="mt-4 max-w-3xl text-sm text-slate-300 sm:text-base">
          This dashboard is organized by operational intent: landing for platform
          understanding, provider workflows for network and node operations, a
          role-aware portfolio for user/provider financial posture, and a full
          end-to-end documentation page.
        </p>

        <div className="mt-6 grid grid-cols-1 gap-3 sm:grid-cols-3">
          <Link href="/provider" className="rounded-2xl border border-cyan-700/30 bg-cyan-900/20 p-4 hover:bg-cyan-900/35">
            <p className="text-sm font-semibold">Provider + Deploy Page</p>
            <p className="mt-1 text-xs text-slate-300">
              Create network, register idle provider, and deploy user jobs to a target network.
            </p>
          </Link>
          <Link href="/portfolio" className="rounded-2xl border border-emerald-700/30 bg-emerald-900/20 p-4 hover:bg-emerald-900/35">
            <p className="text-sm font-semibold">Role-based Portfolio</p>
            <p className="mt-1 text-xs text-slate-300">
              Automatically adapt to user or provider mode based on connected wallet activity.
            </p>
          </Link>
          <Link href="/docs" className="rounded-2xl border border-amber-700/30 bg-amber-900/20 p-4 hover:bg-amber-900/35">
            <p className="text-sm font-semibold">Detailed Docs</p>
            <p className="mt-1 text-xs text-slate-300">
              Full operational and architectural documentation in a production-style format.
            </p>
          </Link>
        </div>
      </section>

      <section className="mt-6 grid grid-cols-2 gap-3 md:grid-cols-6">
        <div className="kpi-pill rounded-xl p-3">
          <p className="text-xs text-slate-400">Networks</p>
          <p className="text-xl font-semibold">{metrics.networks}</p>
        </div>
        <div className="kpi-pill rounded-xl p-3">
          <p className="text-xs text-slate-400">Providers</p>
          <p className="text-xl font-semibold">{metrics.providers}</p>
        </div>
        <div className="kpi-pill rounded-xl p-3">
          <p className="text-xs text-slate-400">Idle Nodes</p>
          <p className="text-xl font-semibold">{metrics.idleNodes}</p>
        </div>
        <div className="kpi-pill rounded-xl p-3">
          <p className="text-xs text-slate-400">Busy Nodes</p>
          <p className="text-xl font-semibold">{metrics.busyNodes}</p>
        </div>
        <div className="kpi-pill rounded-xl p-3">
          <p className="text-xs text-slate-400">Total Jobs</p>
          <p className="text-xl font-semibold">{metrics.jobs}</p>
        </div>
        <div className="kpi-pill rounded-xl p-3">
          <p className="text-xs text-slate-400">Running Jobs</p>
          <p className="text-xl font-semibold">{metrics.runningJobs}</p>
        </div>
      </section>

      <section className="mt-6 grid grid-cols-1 gap-6 xl:grid-cols-2">
        <article className="glass-card rounded-2xl p-5">
          <h2 className="section-title">System Flow Summary</h2>
          <ol className="mt-3 list-decimal space-y-2 pl-5 text-sm text-slate-300">
            <li>Provider creates or joins a network and registers an idle node.</li>
            <li>User submits workload to a selected network with wallet identity.</li>
            <li>Orchestrator assigns idle capacity and executes usage metering lifecycle.</li>
            <li>On-chain settlement and entitlement records become auditable in portfolio.</li>
          </ol>
        </article>

        <article className="glass-card rounded-2xl p-5">
          <div className="flex items-center justify-between gap-3">
            <h2 className="section-title">Live State</h2>
            <button
              onClick={() => void refreshOverview()}
              className="rounded-lg border border-cyan-300/40 bg-cyan-300/20 px-3 py-1.5 text-xs font-semibold hover:bg-cyan-300/30"
            >
              Refresh live data
            </button>
          </div>
          <p className="mt-3 text-sm text-cyan-100/90">{statusMessage}</p>
          <p className="mt-2 text-xs text-slate-400">
            Status is sourced from /networks, /nodes, and /jobs through the orchestrator API proxy.
          </p>
        </article>
      </section>
    </main>
  );
}
