"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import {
  ArrowRight,
  BarChart3,
  CheckCircle2,
  CircleDollarSign,
  Cpu,
  Network,
  ShieldCheck,
  Terminal,
  TrendingUp,
  Wallet,
} from "lucide-react";
import {
  comparisonRows,
  flowSteps,
  landingFeatures,
  landingTerminalSteps,
  testimonialCards,
  tickerStats,
} from "@/lib/dashboard-data";

type TerminalLine = {
  kind: "input" | "output";
  text: string;
};

export default function LandingPage() {
  const [terminalLines, setTerminalLines] = useState<TerminalLine[]>([]);
  const [typingLine, setTypingLine] = useState("initializing network context...");

  useEffect(() => {
    let cancelled = false;

    const sleep = (delay: number) =>
      new Promise<void>((resolve) => {
        window.setTimeout(resolve, delay);
      });

    const playSequence = async () => {
      while (!cancelled) {
        setTerminalLines([]);

        for (const step of landingTerminalSteps) {
          let buffer = "";

          for (const character of step.input) {
            if (cancelled) {
              return;
            }

            buffer += character;
            setTypingLine(buffer);
            await sleep(18);
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
          await sleep(760);
        }

        await sleep(1200);
      }
    };

    void playSequence();

    return () => {
      cancelled = true;
    };
  }, []);

  const tickerItems = useMemo(() => [...tickerStats, ...tickerStats], []);

  return (
    <main className="mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
      <section className="grid gap-6 lg:grid-cols-[1.2fr_0.8fr]">
        <div className="glass-card fade-in-up rounded-[2rem] p-6 sm:p-8">
          <div className="flex flex-wrap items-center gap-3 text-[11px] uppercase tracking-[0.3em] text-slate-400">
            <span className="rounded-full border border-emerald-500/25 bg-emerald-500/10 px-3 py-1 text-emerald-300">
              Devnet live
            </span>
            <span>NodeUnion marketplace</span>
            <span>Monetize idle machines</span>
          </div>

          <h1 className="mt-4 max-w-4xl text-4xl font-semibold tracking-tight text-balance sm:text-6xl">
            Your idle GPU just became an income stream.
          </h1>

          <p className="mt-4 max-w-2xl text-sm leading-6 text-slate-300 sm:text-base">
            NodeUnion turns spare compute into programmable revenue. Providers settle in Stellar assets for useful work, users get elastic capacity, and scheduling stays readable in one clean control plane.
          </p>

          <div className="mt-6 flex flex-wrap gap-3">
            <Link
              href="/networks"
              className="inline-flex items-center gap-2 rounded-full bg-sky-500 px-5 py-3 text-sm font-semibold text-white shadow-lg shadow-sky-500/20 transition hover:bg-sky-400"
            >
              Explore networks <ArrowRight size={16} />
            </Link>
            <Link
              href="/portfolio"
              className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-5 py-3 text-sm font-semibold text-slate-100 transition hover:border-sky-400/40 hover:bg-sky-500/10"
            >
              Open portfolio <Wallet size={16} />
            </Link>
          </div>

          <div className="mt-8 grid gap-3 sm:grid-cols-3">
            {landingFeatures.map((feature, index) => (
              <article
                key={feature.title}
                className={`rounded-2xl border border-white/5 bg-gradient-to-br ${feature.accent} p-4`}
                style={{ animationDelay: `${index * 120}ms` }}
              >
                <div className="flex items-center gap-2 text-sky-300">
                  {index === 0 ? <Network size={18} /> : index === 1 ? <CircleDollarSign size={18} /> : <ShieldCheck size={18} />}
                  <h2 className="font-semibold text-slate-100">{feature.title}</h2>
                </div>
                <p className="mt-2 text-sm leading-6 text-slate-300">{feature.description}</p>
              </article>
            ))}
          </div>
        </div>

        <div className="terminal-glow terminal-window fade-in-up rounded-[2rem] p-4 sm:p-6">
          <div className="flex items-center justify-between border-b border-white/5 pb-4">
            <div className="flex items-center gap-2 text-xs uppercase tracking-[0.3em] text-slate-400">
              <Terminal size={15} className="text-sky-400" />
              Live deployment terminal
            </div>
            <div className="flex items-center gap-2 text-[10px] text-slate-500">
              <span className="h-2 w-2 rounded-full bg-emerald-400" />
              typing simulation
            </div>
          </div>

          <div className="mt-4 min-h-[360px] rounded-[1.5rem] border border-white/5 bg-[#0b1018] p-4 font-mono text-sm leading-6 text-slate-200">
            <div className="space-y-2">
              {terminalLines.map((line, index) => (
                <div key={`${line.kind}-${index}`} className={line.kind === "input" ? "text-sky-300" : "text-emerald-300"}>
                  {line.kind === "input" ? "> " : "✓ "}
                  {line.text}
                </div>
              ))}
              <div className="flex items-center gap-2 text-slate-100">
                <span className="text-sky-300">&gt;</span>
                <span>{typingLine}</span>
                <span className="inline-block h-4 w-2 animate-pulse rounded-sm bg-sky-400/90" />
              </div>
            </div>
          </div>
        </div>
      </section>

      <section className="ticker-mask mt-6 overflow-hidden rounded-[1.5rem] border border-white/5 bg-white/5 px-4 py-3">
        <div className="ticker-track flex w-max items-center gap-6 font-mono text-xs uppercase tracking-[0.28em] text-slate-300">
          {tickerItems.map((item, index) => (
            <div key={`${item.label}-${index}`} className="flex items-center gap-3 whitespace-nowrap">
              <span className="text-slate-500">{item.label}</span>
              <span className="text-emerald-300">{item.value}</span>
            </div>
          ))}
        </div>
      </section>

      <section className="mt-6 grid gap-6 xl:grid-cols-[1.1fr_0.9fr]">
        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <BarChart3 size={15} className="text-sky-400" />
            Platform comparison
          </div>
          <h2 className="mt-3 text-2xl font-semibold tracking-tight">NodeUnion wins on every row.</h2>
          <div className="mt-5 overflow-hidden rounded-2xl border border-white/5">
            <table className="min-w-full text-left text-sm">
              <thead className="bg-white/5 text-slate-400">
                <tr>
                  <th className="px-4 py-3 font-medium">Platform</th>
                  <th className="px-4 py-3 font-medium">Cost / hour</th>
                  <th className="px-4 py-3 font-medium">Setup time</th>
                  <th className="px-4 py-3 font-medium">Idle cost</th>
                  <th className="px-4 py-3 font-medium">Payout model</th>
                </tr>
              </thead>
              <tbody>
                {comparisonRows.map((row) => (
                  <tr
                    key={row.vendor}
                    className={row.best ? "bg-emerald-500/10 text-slate-100" : "border-t border-white/5 text-slate-300"}
                  >
                    <td className="px-4 py-4 font-semibold">
                      <div className="flex items-center gap-2">
                        {row.best ? <CheckCircle2 size={16} className="text-emerald-300" /> : <ShieldCheck size={16} className="text-slate-500" />}
                        {row.vendor}
                      </div>
                    </td>
                    <td className="px-4 py-4">{row.costPerHour}</td>
                    <td className="px-4 py-4">{row.setupTime}</td>
                    <td className="px-4 py-4">{row.idleCost}</td>
                    <td className="px-4 py-4">{row.payoutModel}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </article>

        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <TrendingUp size={15} className="text-emerald-400" />
            How it earns
          </div>
          <div className="mt-4 space-y-3">
            {flowSteps.map((step, index) => (
              <div key={step.title} className="flex gap-3 rounded-2xl border border-white/5 bg-white/5 p-4">
                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-sky-500/15 text-sm font-semibold text-sky-300">
                  {index + 1}
                </div>
                <div>
                  <h3 className="font-semibold text-slate-100">{step.title}</h3>
                  <p className="mt-1 text-sm leading-6 text-slate-300">{step.description}</p>
                </div>
              </div>
            ))}
          </div>
        </article>
      </section>

      <section className="mt-6 grid gap-6 xl:grid-cols-3">
        {testimonialCards.map((card) => (
          <article key={card.author} className="glass-card rounded-[1.5rem] p-5">
            <p className="text-sm leading-6 text-slate-300">“{card.quote}”</p>
            <div className="mt-5 flex items-end justify-between gap-4 border-t border-white/5 pt-4">
              <div>
                <p className="font-semibold text-slate-100">{card.author}</p>
                <p className="text-xs uppercase tracking-[0.2em] text-slate-500">{card.role}</p>
              </div>
              <div className="rounded-full border border-emerald-500/20 bg-emerald-500/10 px-3 py-1 text-xs font-semibold text-emerald-300">
                {card.monthlyEarnings}
              </div>
            </div>
          </article>
        ))}
      </section>

      <section className="mt-6 grid gap-6 lg:grid-cols-4">
        <div className="glass-card rounded-3xl p-5 lg:col-span-1">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <Cpu size={15} className="text-sky-400" />
            Product shape
          </div>
          <div className="mt-4 space-y-4 text-sm leading-6 text-slate-300">
            <p>Production-grade control plane with subtle gradients, glass cards, and monospace metrics.</p>
            <p>Built to sell compute supply, explain billing, and keep operators oriented at a glance.</p>
          </div>
        </div>

        <div className="glass-card rounded-3xl p-5 lg:col-span-3">
          <div className="grid gap-3 sm:grid-cols-3">
            {[
              { label: "Node count", value: "1,248", icon: Cpu },
              { label: "Jobs completed", value: "84,931", icon: CircleDollarSign },
              { label: "SOL paid out", value: "18,420", icon: Wallet },
            ].map((stat) => {
              const Icon = stat.icon;
              return (
                <div key={stat.label} className="rounded-2xl border border-white/5 bg-white/5 p-4">
                  <div className="flex items-center justify-between text-slate-400">
                    <span className="text-xs uppercase tracking-[0.24em]">{stat.label}</span>
                    <Icon size={16} className="text-sky-400" />
                  </div>
                  <p className="mt-3 text-3xl font-semibold tracking-tight">{stat.value}</p>
                </div>
              );
            })}
          </div>
        </div>
      </section>
    </main>
  );
}
