"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import useSWR from "swr";
import { motion } from "framer-motion";
import {
  Activity,
  ArrowRight,
  Boxes,
  CircleDollarSign,
  Clock3,
  Network,
  Rocket,
  Server,
  ShieldCheck,
} from "lucide-react";
import { LiveBadge } from "@/components/live-badge";
import { MetricCard } from "@/components/metric-card";
import { TerminalBlock } from "@/components/terminal-block";
import { MiniEstimator } from "@/components/cost-estimator/MiniEstimator";
import { clampMin, fetchMainSnapshot } from "@/lib/orchestrator-realtime";

type TerminalLine = {
  kind: "input" | "output" | "system";
  text: string;
};

type Snapshot = Awaited<ReturnType<typeof fetchMainSnapshot>>;

const terminalSteps = [
  { input: "$ nodeunion jobs submit --network college-a", output: "Submitting workload payload..." },
  { input: "wallet verified", output: "Wallet accepted. Checking entitlement credits..." },
  { input: "pull ghcr.io/nodeunion/demo:latest", output: "Image pulled. Waiting for a healthy provider..." },
  { input: "assign node provider-node-1", output: "Node assigned. Booting container..." },
  { input: "deploy complete", output: "Container running. Settlement queued in SOL." },
] as const;

const featureCards = [
  {
    icon: Boxes,
    title: "Containerized Jobs",
    description: "Run container workloads with deterministic CPU and RAM limits on live provider nodes.",
  },
  {
    icon: CircleDollarSign,
    title: "Solana Billing",
    description: "Track entitlements, usage, and settlements through the existing on-chain billing layer.",
  },
  {
    icon: Activity,
    title: "Real-time Health",
    description: "Watch heartbeats, node status, and queue pressure update directly from the orchestrator.",
  },
  {
    icon: Network,
    title: "Network Routing",
    description: "Organize providers by network so workloads land on the right capacity pool every time.",
  },
  {
    icon: Server,
    title: "Provider Ops",
    description: "Keep agent state, deployment history, and node assignment visible in one control surface.",
  },
  {
    icon: ShieldCheck,
    title: "Audit-ready Trails",
    description: "Every step from job submission to settlement stays visible for operators and users.",
  },
] as const;

const roleCards = [
  {
    title: "Operator",
    description: "Manage networks, monitor live nodes, and keep the control plane healthy.",
    href: "/networks",
    cta: "View networks",
  },
  {
    title: "Provider",
    description: "Submit workloads, inspect job status, and review deployment history.",
    href: "/provider",
    cta: "Launch workload",
  },
  {
    title: "User",
    description: "Track spend, entitlements, and payout history in the portfolio view.",
    href: "/portfolio",
    cta: "Open portfolio",
  },
] as const;

function formatHours(value: number) {
  return `${value.toLocaleString()} hrs`;
}

function computeMetrics(snapshot?: Snapshot) {
  const nodes = snapshot?.nodes ?? [];
  const jobs = snapshot?.jobs ?? [];
  const networks = snapshot?.networks ?? [];

  const totalNodes = nodes.length;
  const activeJobs = jobs.filter((job) => ["Pending", "Scheduled", "Running"].includes(job.status)).length;
  const networksOnline =
    networks.length > 0 ? new Set(nodes.filter((node) => node.status !== "Offline").map((node) => node.network_id)).size : 0;
  const totalComputeHours = jobs.reduce((sum, job) => {
    const elapsedHours = clampMin((Date.now() / 1000 - job.created_at_epoch_secs) / 3600, 0);
    return sum + Math.max(0.25, Math.min(elapsedHours, 8));
  }, 0);

  return {
    totalNodes,
    activeJobs,
    networksOnline,
    totalComputeHours,
  };
}

export default function LandingPage() {
  const { data, error, isLoading } = useSWR<Snapshot>("/api/main/snapshot", () => fetchMainSnapshot(), {
    refreshInterval: 30000,
    revalidateOnFocus: true,
  });

  const [terminalLines, setTerminalLines] = useState<TerminalLine[]>([]);
  const [typingLine, setTypingLine] = useState("initializing production compute flow...");

  useEffect(() => {
    let cancelled = false;

    const sleep = (delay: number) =>
      new Promise<void>((resolve) => {
        window.setTimeout(resolve, delay);
      });

    const playSequence = async () => {
      while (!cancelled) {
        setTerminalLines([]);

        for (const step of terminalSteps) {
          let buffer = "";

          for (const character of step.input) {
            if (cancelled) {
              return;
            }

            buffer += character;
            setTypingLine(buffer);
            await sleep(16);
          }

          if (cancelled) {
            return;
          }

          setTerminalLines((current) => [
            ...current,
            { kind: "input", text: step.input },
            { kind: "output", text: step.output },
          ]);
          setTypingLine("waiting for agent heartbeat...");
          await sleep(700);
        }

        await sleep(1000);
      }
    };

    void playSequence();

    return () => {
      cancelled = true;
    };
  }, []);

  const metrics = useMemo(() => computeMetrics(data), [data]);

  return (
    <main className="mx-auto flex w-full max-w-7xl flex-col gap-8 px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
      <section className="grid gap-6 xl:grid-cols-[1.06fr_0.94fr]">
        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.35, ease: "easeOut" }}
          className="glass-card rounded-[2rem] p-6 sm:p-8"
        >
          <div className="flex flex-wrap items-center gap-3 text-[11px] uppercase tracking-[0.32em] text-slate-400">
            <LiveBadge />
            <span>Snapshot refresh: 30s</span>
            <span>{error ? "snapshot offline" : "API proxy connected"}</span>
          </div>

          <h1 className="mt-5 max-w-3xl text-4xl font-semibold tracking-tight text-balance sm:text-6xl">
            Decentralized Compute, Production Grade.
          </h1>

          <p className="mt-5 max-w-2xl text-sm leading-7 text-slate-300 sm:text-base">
            NodeUnion is the web control plane for a decentralized compute marketplace. Operators manage networks,
            providers launch workloads, and users track earnings and settlement without leaving the dashboard.
          </p>

          <div className="mt-6 flex flex-wrap gap-3">
            <Link
              href="/provider"
              className="inline-flex items-center gap-2 rounded-full bg-indigo-500 px-5 py-3 text-sm font-semibold text-white shadow-lg shadow-indigo-500/20 transition hover:bg-indigo-400"
            >
              Launch Workload <ArrowRight size={16} />
            </Link>
            <Link
              href="/onboarding"
              className="inline-flex items-center gap-2 rounded-full border border-indigo-400/40 bg-indigo-500/10 px-5 py-3 text-sm font-semibold text-indigo-200 transition hover:bg-indigo-500/20"
            >
              Become a Provider <ArrowRight size={16} />
            </Link>
            <Link
              href="/networks"
              className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-5 py-3 text-sm font-semibold text-slate-100 transition hover:border-cyan-400/40 hover:bg-cyan-500/10"
            >
              View Network <Network size={16} />
            </Link>
          </div>

          <div className="mt-8 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
            <MetricCard
              label="Total Nodes"
              value={isLoading ? "—" : metrics.totalNodes.toLocaleString()}
              delta={error ? "snapshot failed" : "from /api/main/snapshot"}
              icon={<Server size={16} className="text-cyan-300" />}
            />
            <MetricCard
              label="Active Jobs"
              value={isLoading ? "—" : metrics.activeJobs.toLocaleString()}
              delta="live queue depth"
              icon={<Rocket size={16} className="text-indigo-300" />}
            />
            <MetricCard
              label="Networks Online"
              value={isLoading ? "—" : metrics.networksOnline.toLocaleString()}
              delta="currently serving traffic"
              icon={<Network size={16} className="text-cyan-300" />}
            />
            <MetricCard
              label="Total Compute Hours"
              value={isLoading ? "—" : formatHours(metrics.totalComputeHours)}
              delta="rolling estimate"
              icon={<Clock3 size={16} className="text-indigo-300" />}
            />
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.35, ease: "easeOut", delay: 0.05 }}
        >
          <TerminalBlock
            title="CLI job submission"
            subtitle="Typewriter preview of a real workload flow"
            lines={terminalLines}
            typingLine={typingLine}
            footer={<span className="text-xs uppercase tracking-[0.24em] text-slate-500">Web to orchestrator</span>}
          />
        </motion.div>
      </section>

      <motion.section
        initial={{ opacity: 0, y: 12 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true, amount: 0.25 }}
        transition={{ duration: 0.35, ease: "easeOut" }}
        className="glass-card rounded-[2rem] p-6 sm:p-8"
      >
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p className="text-[11px] uppercase tracking-[0.32em] text-cyan-300">How It Works</p>
            <h2 className="mt-2 text-2xl font-semibold tracking-tight text-slate-100">
              From node registration to payment settlement.
            </h2>
          </div>
          <p className="max-w-xl text-sm leading-6 text-slate-400">
            The dashboard mirrors the live product flow: register capacity, submit a job, and let the orchestrator
            handle assignment and settlement.
          </p>
        </div>

        <div className="relative mt-8">
          <motion.div
            initial={{ scaleX: 0 }}
            whileInView={{ scaleX: 1 }}
            viewport={{ once: true, amount: 0.25 }}
            transition={{ duration: 0.35, ease: "easeOut" }}
            className="absolute left-6 right-6 top-10 hidden h-px origin-left bg-gradient-to-r from-transparent via-white/20 to-transparent lg:block"
          />

          <div className="grid gap-4 lg:grid-cols-3">
            {[
              {
                step: "01",
                title: "Register Node",
                description: "Providers onboard agent machines and publish live capacity into a network.",
              },
              {
                step: "02",
                title: "Submit Job",
                description: "Users launch a container workload with CPU, RAM, and port controls.",
              },
              {
                step: "03",
                title: "Settle Payment",
                description: "Compute usage is tracked, billed, and settled through the Solana billing path.",
              },
            ].map((item, index) => (
              <motion.article
                key={item.step}
                initial={{ opacity: 0, y: 10 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, amount: 0.25 }}
                transition={{ duration: 0.3, ease: "easeOut", delay: index * 0.05 }}
                className="rounded-3xl border border-white/10 bg-white/5 p-5"
              >
                <div className="flex items-center gap-3">
                  <span className="metric-value text-3xl text-indigo-300">{item.step}</span>
                  <span className="h-px flex-1 bg-white/10" />
                  <span className="rounded-full border border-cyan-400/20 bg-cyan-400/10 px-3 py-1 text-[11px] uppercase tracking-[0.22em] text-cyan-300">
                    Step
                  </span>
                </div>
                <h3 className="mt-4 text-lg font-semibold text-slate-100">{item.title}</h3>
                <p className="mt-2 text-sm leading-6 text-slate-300">{item.description}</p>
              </motion.article>
            ))}
          </div>
        </div>
      </motion.section>

      <MiniEstimator />

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {featureCards.map((feature, index) => {
          const Icon = feature.icon;

          return (
            <motion.article
              key={feature.title}
              initial={{ opacity: 0, y: 12 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, amount: 0.2 }}
              transition={{ duration: 0.3, ease: "easeOut", delay: index * 0.03 }}
              className="glass-card rounded-[1.75rem] p-5"
            >
              <div className="flex items-center gap-3">
                <div className="rounded-2xl border border-white/10 bg-white/5 p-3 text-cyan-300">
                  <Icon size={18} />
                </div>
                <h3 className="text-base font-semibold text-slate-100">{feature.title}</h3>
              </div>
              <p className="mt-4 text-sm leading-6 text-slate-300">{feature.description}</p>
            </motion.article>
          );
        })}
      </section>

      <motion.section
        initial={{ opacity: 0, y: 12 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true, amount: 0.2 }}
        transition={{ duration: 0.35, ease: "easeOut" }}
        className="glass-card rounded-[2rem] p-6 sm:p-8"
      >
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="text-[11px] uppercase tracking-[0.32em] text-cyan-300">Roles</p>
            <h2 className="mt-2 text-2xl font-semibold tracking-tight text-slate-100">
              One product, three entry points.
            </h2>
          </div>
          <p className="max-w-xl text-sm leading-6 text-slate-400">
            The interface keeps each role focused on its own workflow while staying inside one shared dashboard.
          </p>
        </div>

        <div className="mt-6 grid gap-4 xl:grid-cols-3">
          {roleCards.map((role) => (
            <article key={role.title} className="rounded-[1.5rem] border border-white/10 bg-white/5 p-5">
              <p className="text-[11px] uppercase tracking-[0.32em] text-slate-500">{role.title}</p>
              <p className="mt-3 text-sm leading-6 text-slate-300">{role.description}</p>
              <Link
                href={role.href}
                className="mt-5 inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-4 py-2 text-sm font-semibold text-slate-100 transition hover:border-cyan-400/40 hover:bg-cyan-500/10"
              >
                {role.cta} <ArrowRight size={15} />
              </Link>
            </article>
          ))}
        </div>
      </motion.section>

      <footer className="flex flex-col gap-4 border-t border-white/10 pt-6 text-sm text-slate-400 sm:flex-row sm:items-center sm:justify-between">
        <p className="font-mono text-xs uppercase tracking-[0.28em] text-slate-500">
          NodeUnion dashboard · decentralized compute marketplace
        </p>
        <div className="flex flex-wrap gap-3">
          <Link href="/networks" className="nav-link">
            Networks
          </Link>
          <Link href="/provider" className="nav-link">
            Provider
          </Link>
          <Link href="/portfolio" className="nav-link">
            Portfolio
          </Link>
          <Link href="/docs" className="nav-link">
            Docs
          </Link>
        </div>
      </footer>
    </main>
  );
}