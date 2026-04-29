"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState, type ReactNode } from "react";
import { motion } from "framer-motion";
import {
  BookOpen,
  ChevronDown,
  ExternalLink,
  LayoutDashboard,
  Network,
  PieChart,
  Rocket,
  Server,
  Wrench,
} from "lucide-react";
import ThemeToggle from "@/components/theme-toggle";
import { LiveBadge } from "@/components/live-badge";

const navigationItems = [
  { href: "/", label: "Landing", description: "Product overview", icon: LayoutDashboard },
  { href: "/networks", label: "Networks", description: "Live topology", icon: Network },
  { href: "/onboarding", label: "Onboarding", description: "Become provider", icon: Rocket },
  { href: "/provider", label: "Provider", description: "Launch workloads", icon: Rocket },
  { href: "/portfolio", label: "Portfolio", description: "Earnings & usage", icon: PieChart },
  { href: "/docs", label: "Docs", description: "Setup guide", icon: BookOpen },
] as const;

const toolsItems = [
  { href: "/tools/cli-builder", label: "CLI Builder" },
  { href: "/tools/cost-estimator", label: "Cost Estimator" },
] as const;

export function AppShell({ children }: { children: ReactNode }) {
  const pathname = usePathname();
  const [toolsOpen, setToolsOpen] = useState(false);

  return (
    <div className="app-shell-frame">
      <aside className="shell-sidebar hidden lg:flex lg:flex-col">
        <div className="flex items-start justify-between border-b border-white/10 px-6 py-6">
          <div>
            <p className="font-mono text-xs uppercase tracking-[0.35em] text-slate-100">NodeUnion</p>
            <p className="mt-2 max-w-44 text-sm leading-6 text-slate-400">
              Infrastructure control plane for decentralized compute.
            </p>
          </div>
          <LiveBadge />
        </div>

        <nav className="flex flex-1 flex-col gap-2 px-4 py-5">
          <p className="shell-nav-section px-3 text-[11px] uppercase tracking-[0.28em] text-slate-500">
            Workspace
          </p>
          {navigationItems.map((item) => {
            const Icon = item.icon;
            const active = pathname === item.href;

            return (
              <Link key={item.href} href={item.href} className="shell-nav-link" data-active={active}>
                <span className="flex min-w-0 items-start gap-3">
                  <span className="mt-0.5 rounded-xl border border-white/10 bg-white/5 p-2 text-sky-300">
                    <Icon size={16} />
                  </span>
                  <span className="min-w-0">
                    <span className="block text-sm font-medium text-slate-100">{item.label}</span>
                    <span className="block text-xs leading-5 text-slate-400">{item.description}</span>
                  </span>
                </span>
                <ExternalLink size={14} className="shrink-0 text-slate-500" />
              </Link>
            );
          })}

          <p className="shell-nav-section mt-4 px-3 text-[11px] uppercase tracking-[0.28em] text-slate-500">
            Tools
          </p>
          {toolsItems.map((tool) => (
            <Link
              key={tool.href}
              href={tool.href}
              className="shell-nav-link"
              data-active={pathname === tool.href}
            >
              <span className="flex min-w-0 items-start gap-3">
                <span className="mt-0.5 rounded-xl border border-white/10 bg-white/5 p-2 text-sky-300">
                  <Wrench size={16} />
                </span>
                <span className="min-w-0">
                  <span className="block text-sm font-medium text-slate-100">{tool.label}</span>
                  <span className="block text-xs leading-5 text-slate-400">Developer tooling</span>
                </span>
              </span>
              <ExternalLink size={14} className="shrink-0 text-slate-500" />
            </Link>
          ))}
        </nav>

        <div className="border-t border-white/10 p-6">
          <div className="rounded-3xl border border-white/10 bg-white/5 p-4">
            <div className="flex items-center justify-between gap-3">
              <div>
                <p className="text-xs uppercase tracking-[0.28em] text-slate-500">Backend</p>
                <p className="mt-2 text-sm font-medium text-slate-100">Orchestrator connected</p>
              </div>
              <Server size={18} className="text-cyan-300" />
            </div>
            <p className="mt-3 text-sm leading-6 text-slate-400">
              API requests continue to proxy through the same orchestrator and main routes.
            </p>
          </div>
        </div>
      </aside>

      <div className="shell-main flex min-w-0 flex-1 flex-col">
        <header className="shell-topbar sticky top-0 z-40">
          <div className="flex items-center justify-between gap-4 px-4 py-4 sm:px-6 lg:px-8">
            <div className="min-w-0 lg:hidden">
              <p className="font-mono text-xs uppercase tracking-[0.3em] text-slate-100">NodeUnion</p>
              <p className="mt-1 text-[11px] uppercase tracking-[0.28em] text-slate-500">
                Decentralized compute marketplace
              </p>
            </div>
            <div className="hidden lg:block">
              <div className="flex items-center gap-3 text-sm text-slate-400">
                <span>Web dashboard for operators, providers, and users</span>
                <div className="relative">
                  <button
                    type="button"
                    onClick={() => setToolsOpen((value) => !value)}
                    className="inline-flex items-center gap-1 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-slate-200 hover:bg-white/10"
                  >
                    Tools <ChevronDown size={14} />
                  </button>

                  {toolsOpen ? (
                    <div className="absolute right-0 top-9 w-44 rounded-xl border border-white/10 bg-[#0a0a0f] p-2 shadow-xl">
                      {toolsItems.map((tool) => (
                        <Link
                          key={tool.href}
                          href={tool.href}
                          onClick={() => setToolsOpen(false)}
                          className="block rounded-md px-3 py-2 text-xs text-slate-200 hover:bg-white/10"
                        >
                          {tool.label}
                        </Link>
                      ))}
                    </div>
                  ) : null}
                </div>
              </div>
            </div>
            <div className="flex items-center gap-3">
              <LiveBadge />
              <ThemeToggle />
            </div>
          </div>

          <nav className="flex gap-2 overflow-x-auto px-4 pb-4 sm:px-6 lg:hidden">
            {navigationItems.map((item) => {
              const active = pathname === item.href;

              return (
                <Link key={item.href} href={item.href} className="nav-link whitespace-nowrap" data-active={active}>
                  {item.label}
                </Link>
              );
            })}
            {toolsItems.map((tool) => (
              <Link
                key={tool.href}
                href={tool.href}
                className="nav-link whitespace-nowrap"
                data-active={pathname === tool.href}
              >
                {tool.label}
              </Link>
            ))}
          </nav>
        </header>

        <main className="shell-content flex-1">
          <motion.div
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.35, ease: "easeOut" }}
          >
            {children}
          </motion.div>
        </main>
      </div>
    </div>
  );
}