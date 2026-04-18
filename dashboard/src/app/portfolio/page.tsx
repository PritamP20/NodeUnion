"use client";

import { useEffect, useMemo, useState } from "react";
import {
  Entitlement,
  Job,
  Network,
  Node,
  Settlement,
  connectWallet,
  disconnectWallet,
  fetchOverview,
  fetchWalletPortfolio,
  shortenWallet,
} from "@/lib/orchestrator";

type PortfolioMode = "auto" | "provider" | "user";

export default function PortfolioPage() {
  const [walletAddress, setWalletAddress] = useState("");
  const [walletConnected, setWalletConnected] = useState(false);
  const [mode, setMode] = useState<PortfolioMode>("auto");
  const [statusMessage, setStatusMessage] = useState("Connect wallet to build your portfolio view");

  const [networks, setNetworks] = useState<Network[]>([]);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [jobs, setJobs] = useState<Job[]>([]);

  const [entitlements, setEntitlements] = useState<Entitlement[]>([]);
  const [settlements, setSettlements] = useState<Settlement[]>([]);

  async function refreshOverview() {
    try {
      const overview = await fetchOverview();
      setNetworks(overview.networks);
      setNodes(overview.nodes);
      setJobs(overview.jobs);
    } catch (error) {
      setStatusMessage(`Overview refresh failed: ${(error as Error).message}`);
    }
  }

  useEffect(() => {
    void refreshOverview();
  }, []);

  async function onConnectWallet() {
    try {
      const wallet = await connectWallet();
      setWalletAddress(wallet);
      setWalletConnected(true);

      await refreshOverview();
      const portfolio = await fetchWalletPortfolio(wallet);
      setEntitlements(portfolio.entitlements);
      setSettlements(portfolio.settlements);
      setStatusMessage(`Portfolio loaded for ${shortenWallet(wallet)}`);
    } catch (error) {
      setStatusMessage(`Wallet connection failed: ${(error as Error).message}`);
    }
  }

  async function onDisconnectWallet() {
    try {
      await disconnectWallet();
    } finally {
      setWalletAddress("");
      setWalletConnected(false);
      setEntitlements([]);
      setSettlements([]);
      setStatusMessage("Wallet disconnected");
    }
  }

  const providerNodes = useMemo(
    () => nodes.filter((node) => node.provider_wallet === walletAddress),
    [nodes, walletAddress],
  );

  const providerNodeIds = useMemo(
    () => new Set(providerNodes.map((node) => node.node_id)),
    [providerNodes],
  );

  const providerJobs = useMemo(
    () => jobs.filter((job) => job.assigned_node_id && providerNodeIds.has(job.assigned_node_id)),
    [jobs, providerNodeIds],
  );

  const providerNetworkIds = useMemo(
    () => new Set(providerNodes.map((node) => node.network_id)),
    [providerNodes],
  );

  const providerNetworks = useMemo(
    () => networks.filter((network) => providerNetworkIds.has(network.network_id)),
    [networks, providerNetworkIds],
  );

  const userCredits = useMemo(
    () => entitlements.reduce((sum, item) => sum + item.bought_units, 0),
    [entitlements],
  );

  const userConsumed = useMemo(
    () => entitlements.reduce((sum, item) => sum + item.used_units, 0),
    [entitlements],
  );

  const inferredRole = useMemo(() => {
    if (!walletAddress) return "none";
    if (providerNodes.length > 0) return "provider";
    return "user";
  }, [providerNodes, walletAddress]);

  const effectiveRole = mode === "auto" ? inferredRole : mode;

  return (
    <main className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <header className="glass-card rounded-3xl p-6 sm:p-8">
        <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div>
            <p className="font-mono text-xs tracking-[0.2em] text-cyan-200/70">
              ROLE-BASED PORTFOLIO
            </p>
            <h1 className="mt-2 text-3xl font-semibold tracking-tight sm:text-4xl">
              Portfolio view for network providers and users
            </h1>
            <p className="mt-2 max-w-3xl text-sm text-slate-300">
              In auto mode, role is inferred from wallet activity. If wallet has
              registered provider nodes, provider perspective is shown. Otherwise
              user portfolio is shown using entitlements and settlement records.
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <select
              value={mode}
              onChange={(event) => setMode(event.target.value as PortfolioMode)}
              className="rounded-xl border border-cyan-300/40 bg-cyan-900/20 px-3 py-2 text-sm"
            >
              <option value="auto">Auto role</option>
              <option value="provider">Force provider view</option>
              <option value="user">Force user view</option>
            </select>
            {!walletConnected ? (
              <button
                onClick={() => void onConnectWallet()}
                className="rounded-xl bg-amber-300 px-4 py-2 text-sm font-semibold text-slate-900 hover:bg-amber-200"
              >
                Connect Wallet
              </button>
            ) : (
              <button
                onClick={() => void onDisconnectWallet()}
                className="rounded-xl border border-rose-300/60 bg-rose-400/20 px-4 py-2 text-sm font-semibold text-rose-100 hover:bg-rose-400/30"
              >
                Disconnect {shortenWallet(walletAddress)}
              </button>
            )}
          </div>
        </div>

        <div className="mt-4 grid grid-cols-2 gap-3 md:grid-cols-4">
          <div className="kpi-pill rounded-xl p-3">
            <p className="text-xs text-slate-400">Detected Role</p>
            <p className="text-xl font-semibold capitalize">{inferredRole}</p>
          </div>
          <div className="kpi-pill rounded-xl p-3">
            <p className="text-xs text-slate-400">Effective View</p>
            <p className="text-xl font-semibold capitalize">{effectiveRole}</p>
          </div>
          <div className="kpi-pill rounded-xl p-3">
            <p className="text-xs text-slate-400">Credits Bought</p>
            <p className="text-xl font-semibold">{userCredits}</p>
          </div>
          <div className="kpi-pill rounded-xl p-3">
            <p className="text-xs text-slate-400">Credits Used</p>
            <p className="text-xl font-semibold">{userConsumed}</p>
          </div>
        </div>

        <p className="mt-4 text-sm text-cyan-100/90">{statusMessage}</p>
      </header>

      {!walletAddress ? (
        <section className="mt-6 glass-card rounded-2xl p-5">
          <p className="text-sm text-slate-300">
            Connect a wallet to display either provider performance or user financial portfolio.
          </p>
        </section>
      ) : effectiveRole === "provider" ? (
        <section className="mt-6 grid grid-cols-1 gap-6 xl:grid-cols-3">
          <article className="glass-card rounded-2xl p-5">
            <h2 className="section-title">Provider Networks</h2>
            <div className="mt-3 space-y-2">
              {providerNetworks.length === 0 ? (
                <p className="text-sm text-slate-300">No network ownership detected for this wallet.</p>
              ) : (
                providerNetworks.map((network) => (
                  <div
                    key={network.network_id}
                    className="rounded-xl border border-cyan-900/40 bg-slate-950/50 p-3"
                  >
                    <p className="font-semibold">{network.name}</p>
                    <p className="font-mono text-xs text-slate-400">{network.network_id}</p>
                  </div>
                ))
              )}
            </div>
          </article>

          <article className="glass-card rounded-2xl p-5">
            <h2 className="section-title">Provider Nodes</h2>
            <div className="mt-3 space-y-2">
              {providerNodes.length === 0 ? (
                <p className="text-sm text-slate-300">No provider nodes registered by this wallet.</p>
              ) : (
                providerNodes.map((node) => (
                  <div
                    key={node.node_id}
                    className="rounded-xl border border-cyan-900/40 bg-slate-950/50 p-3"
                  >
                    <p className="font-mono text-xs text-slate-300">{node.node_id}</p>
                    <p className="mt-1 text-xs text-slate-400">
                      {node.network_id} · {node.region ?? "unknown-region"} · {node.is_idle ? "Idle" : "Busy"}
                    </p>
                  </div>
                ))
              )}
            </div>
          </article>

          <article className="glass-card rounded-2xl p-5">
            <h2 className="section-title">Assigned Jobs</h2>
            <div className="mt-3 space-y-2">
              {providerJobs.length === 0 ? (
                <p className="text-sm text-slate-300">No jobs currently assigned to provider nodes.</p>
              ) : (
                providerJobs.map((job) => (
                  <div
                    key={job.job_id}
                    className="rounded-xl border border-cyan-900/40 bg-slate-950/50 p-3"
                  >
                    <p className="font-mono text-xs text-slate-300">{job.job_id}</p>
                    <p className="mt-1 text-xs text-slate-400">
                      {job.network_id} · {job.image} · {job.status}
                    </p>
                  </div>
                ))
              )}
            </div>
          </article>
        </section>
      ) : (
        <section className="mt-6 grid grid-cols-1 gap-6 xl:grid-cols-2">
          <article className="glass-card rounded-2xl p-5">
            <h2 className="section-title">User Entitlements</h2>
            <div className="mt-3 space-y-2">
              {entitlements.length === 0 ? (
                <p className="text-sm text-slate-300">No entitlement records found for this wallet.</p>
              ) : (
                entitlements.map((entitlement) => (
                  <div
                    key={entitlement.entitlement_id}
                    className="rounded-xl border border-cyan-900/40 bg-slate-950/50 p-3"
                  >
                    <p className="font-semibold">{entitlement.network_id}</p>
                    <p className="mt-1 text-xs text-slate-300">
                      Bought {entitlement.bought_units} · Used {entitlement.used_units} · Remaining {entitlement.bought_units - entitlement.used_units}
                    </p>
                    <p className="mt-1 font-mono text-xs text-slate-400">
                      Escrow Tx {entitlement.escrow_tx_hash ?? "-"}
                    </p>
                  </div>
                ))
              )}
            </div>
          </article>

          <article className="glass-card rounded-2xl p-5">
            <h2 className="section-title">User Settlement Ledger</h2>
            <div className="mt-3 space-y-2">
              {settlements.length === 0 ? (
                <p className="text-sm text-slate-300">No settlement records found for this wallet.</p>
              ) : (
                settlements.map((settlement) => (
                  <div
                    key={settlement.settlement_id}
                    className="rounded-xl border border-cyan-900/40 bg-slate-950/50 p-3"
                  >
                    <p className="font-mono text-xs text-slate-300">{settlement.settlement_id}</p>
                    <p className="mt-1 text-xs text-slate-300">
                      {settlement.network_id} · job {settlement.job_id} · units {settlement.units_metered} · tokens {settlement.amount_tokens}
                    </p>
                    <p className="mt-1 text-xs text-slate-400">
                      {settlement.tx_status ?? "pending"} · {settlement.tx_hash ?? "no tx hash"}
                    </p>
                  </div>
                ))
              )}
            </div>
          </article>
        </section>
      )}
    </main>
  );
}
