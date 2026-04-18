"use client";

import { FormEvent, useMemo, useState } from "react";
import {
  Job,
  Network,
  Node,
  connectWallet,
  disconnectWallet,
  fetchJson,
  fetchOverview,
  shortenWallet,
} from "@/lib/orchestrator";

export default function ProviderAndDeployPage() {
  const [networks, setNetworks] = useState<Network[]>([]);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [jobs, setJobs] = useState<Job[]>([]);
  const [statusMessage, setStatusMessage] = useState("Ready");

  const [walletAddress, setWalletAddress] = useState("");
  const [walletConnected, setWalletConnected] = useState(false);

  const [newNetworkId, setNewNetworkId] = useState("");
  const [newNetworkName, setNewNetworkName] = useState("");
  const [newNetworkDescription, setNewNetworkDescription] = useState("");

  const [nodeId, setNodeId] = useState("");
  const [nodeNetworkId, setNodeNetworkId] = useState("");
  const [nodeAgentUrl, setNodeAgentUrl] = useState("http://127.0.0.1:8090");
  const [nodeRegion, setNodeRegion] = useState("us-east-1");
  const [providerWallet, setProviderWallet] = useState("");

  const [jobNetworkId, setJobNetworkId] = useState("");
  const [jobImage, setJobImage] = useState("alpine:3.20");
  const [jobCommand, setJobCommand] = useState("echo hello-from-nodeunion");

  const networkOptions = useMemo(
    () => networks.map((network) => network.network_id),
    [networks],
  );

  async function refreshAll() {
    try {
      const overview = await fetchOverview();
      setNetworks(overview.networks);
      setNodes(overview.nodes);
      setJobs(overview.jobs);
      setStatusMessage("Provider and deployment data refreshed");
    } catch (error) {
      setStatusMessage(`Refresh failed: ${(error as Error).message}`);
    }
  }

  async function onConnectWallet() {
    try {
      const wallet = await connectWallet();
      setWalletAddress(wallet);
      setWalletConnected(true);
      setProviderWallet(wallet);
      setStatusMessage(`Wallet connected: ${shortenWallet(wallet)}`);
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
      setProviderWallet("");
      setStatusMessage("Wallet disconnected");
    }
  }

  async function onCreateNetwork(event: FormEvent) {
    event.preventDefault();
    try {
      const result = await fetchJson<{ message: string }>("/networks/create", {
        method: "POST",
        body: JSON.stringify({
          network_id: newNetworkId.trim(),
          name: newNetworkName.trim(),
          description: newNetworkDescription.trim() || undefined,
        }),
      });

      setNewNetworkId("");
      setNewNetworkName("");
      setNewNetworkDescription("");
      await refreshAll();
      setStatusMessage(result.message || "Network created successfully");
    } catch (error) {
      setStatusMessage(`Create network failed: ${(error as Error).message}`);
    }
  }

  async function onRegisterProvider(event: FormEvent) {
    event.preventDefault();
    try {
      const result = await fetchJson<{ message: string }>("/nodes/register", {
        method: "POST",
        body: JSON.stringify({
          node_id: nodeId.trim(),
          network_id: nodeNetworkId.trim(),
          agent_url: nodeAgentUrl.trim(),
          provider_wallet: providerWallet.trim() || undefined,
          region: nodeRegion.trim() || undefined,
          labels: { source: "provider-page" },
        }),
      });

      setNodeId("");
      await refreshAll();
      setStatusMessage(result.message || "Provider registered");
    } catch (error) {
      setStatusMessage(`Provider registration failed: ${(error as Error).message}`);
    }
  }

  async function onDeployUserJob(event: FormEvent) {
    event.preventDefault();
    if (!walletAddress) {
      setStatusMessage("Connect user wallet before deploying a workload.");
      return;
    }

    try {
      const command = jobCommand
        .split(" ")
        .map((segment) => segment.trim())
        .filter(Boolean);

      const result = await fetchJson<{ job_id: string; message?: string }>(
        "/jobs/submit",
        {
          method: "POST",
          body: JSON.stringify({
            network_id: jobNetworkId.trim(),
            user_wallet: walletAddress,
            image: jobImage.trim(),
            command: command.length > 0 ? command : undefined,
            cpu_limit: 0.25,
            ram_limit_mb: 128,
          }),
        },
      );

      await refreshAll();
      setStatusMessage(result.message || `Job deployed: ${result.job_id}`);
    } catch (error) {
      setStatusMessage(`Deploy job failed: ${(error as Error).message}`);
    }
  }

  const idleNodes = nodes.filter((node) => node.is_idle).length;

  return (
    <main className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <header className="glass-card rounded-3xl p-6 sm:p-8">
        <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div>
            <p className="font-mono text-xs tracking-[0.2em] text-cyan-200/70">
              PROVIDER + USER DEPLOY WORKFLOWS
            </p>
            <h1 className="mt-2 text-3xl font-semibold tracking-tight sm:text-4xl">
              Idle compute provider operations and user deployment
            </h1>
            <p className="mt-2 max-w-3xl text-sm text-slate-300">
              This page combines all operational forms in one place: network
              creation, provider registration for idle capacity, and user workload
              deployment into a desired network.
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <button
              onClick={() => void refreshAll()}
              className="rounded-xl border border-cyan-300/40 bg-cyan-300/20 px-4 py-2 text-sm font-semibold hover:bg-cyan-300/30"
            >
              Refresh
            </button>
            {!walletConnected ? (
              <button
                onClick={() => void onConnectWallet()}
                className="rounded-xl bg-amber-300 px-4 py-2 text-sm font-semibold text-slate-900 hover:bg-amber-200"
              >
                Connect User Wallet
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
            <p className="text-xs text-slate-400">Networks</p>
            <p className="text-xl font-semibold">{networks.length}</p>
          </div>
          <div className="kpi-pill rounded-xl p-3">
            <p className="text-xs text-slate-400">Providers</p>
            <p className="text-xl font-semibold">{nodes.length}</p>
          </div>
          <div className="kpi-pill rounded-xl p-3">
            <p className="text-xs text-slate-400">Idle Providers</p>
            <p className="text-xl font-semibold">{idleNodes}</p>
          </div>
          <div className="kpi-pill rounded-xl p-3">
            <p className="text-xs text-slate-400">Jobs</p>
            <p className="text-xl font-semibold">{jobs.length}</p>
          </div>
        </div>

        <p className="mt-4 text-sm text-cyan-100/90">{statusMessage}</p>
      </header>

      <section className="mt-6 grid grid-cols-1 gap-6 xl:grid-cols-3">
        <form onSubmit={onCreateNetwork} className="glass-card rounded-2xl p-5">
          <h2 className="section-title">1. Create Network</h2>
          <p className="mt-1 text-xs text-slate-400">
            Register a workload network first so providers and users can target it.
          </p>
          <input
            value={newNetworkId}
            onChange={(event) => setNewNetworkId(event.target.value)}
            placeholder="college-a"
            className="mt-3 mb-2 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 text-sm"
            required
          />
          <input
            value={newNetworkName}
            onChange={(event) => setNewNetworkName(event.target.value)}
            placeholder="College A Network"
            className="mb-2 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 text-sm"
            required
          />
          <input
            value={newNetworkDescription}
            onChange={(event) => setNewNetworkDescription(event.target.value)}
            placeholder="Optional description"
            className="mb-3 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 text-sm"
          />
          <button className="w-full rounded-lg bg-emerald-400 px-3 py-2 text-sm font-semibold text-slate-950 hover:bg-emerald-300">
            Register Network
          </button>
        </form>

        <form onSubmit={onRegisterProvider} className="glass-card rounded-2xl p-5">
          <h2 className="section-title">2. Register Idle Provider</h2>
          <p className="mt-1 text-xs text-slate-400">
            Attach node identity, region, and provider wallet to a selected network.
          </p>
          <input
            value={nodeId}
            onChange={(event) => setNodeId(event.target.value)}
            placeholder="provider-node-1"
            className="mt-3 mb-2 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 text-sm"
            required
          />
          <input
            value={nodeAgentUrl}
            onChange={(event) => setNodeAgentUrl(event.target.value)}
            placeholder="http://127.0.0.1:8090"
            className="mb-2 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 text-sm"
            required
          />
          <select
            value={nodeNetworkId}
            onChange={(event) => setNodeNetworkId(event.target.value)}
            className="mb-2 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 text-sm"
            required
          >
            <option value="">Select network</option>
            {networkOptions.map((networkId) => (
              <option key={networkId} value={networkId}>
                {networkId}
              </option>
            ))}
          </select>
          <input
            value={providerWallet}
            onChange={(event) => setProviderWallet(event.target.value)}
            placeholder="Provider wallet"
            className="mb-2 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 font-mono text-xs"
          />
          <input
            value={nodeRegion}
            onChange={(event) => setNodeRegion(event.target.value)}
            placeholder="us-east-1"
            className="mb-3 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 text-sm"
          />
          <button className="w-full rounded-lg bg-orange-300 px-3 py-2 text-sm font-semibold text-slate-900 hover:bg-orange-200">
            Register Provider
          </button>
        </form>

        <form onSubmit={onDeployUserJob} className="glass-card rounded-2xl p-5">
          <h2 className="section-title">3. Deploy User Job</h2>
          <p className="mt-1 text-xs text-slate-400">
            Submit workload to the desired network using connected wallet identity.
          </p>
          <select
            value={jobNetworkId}
            onChange={(event) => setJobNetworkId(event.target.value)}
            className="mt-3 mb-2 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 text-sm"
            required
          >
            <option value="">Select network</option>
            {networkOptions.map((networkId) => (
              <option key={networkId} value={networkId}>
                {networkId}
              </option>
            ))}
          </select>
          <input
            value={walletAddress}
            onChange={(event) => setWalletAddress(event.target.value)}
            placeholder="User wallet"
            className="mb-2 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 font-mono text-xs"
            required
          />
          <input
            value={jobImage}
            onChange={(event) => setJobImage(event.target.value)}
            placeholder="alpine:3.20"
            className="mb-2 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 text-sm"
            required
          />
          <input
            value={jobCommand}
            onChange={(event) => setJobCommand(event.target.value)}
            placeholder="echo hello-from-nodeunion"
            className="mb-3 w-full rounded-lg border border-cyan-900/60 bg-slate-950/50 px-3 py-2 text-sm"
          />
          <button className="w-full rounded-lg bg-cyan-300 px-3 py-2 text-sm font-semibold text-slate-900 hover:bg-cyan-200">
            Deploy Job
          </button>
        </form>
      </section>

      <section className="mt-6 grid grid-cols-1 gap-6 xl:grid-cols-2">
        <article className="glass-card rounded-2xl p-5">
          <h3 className="section-title">Available Networks</h3>
          <div className="mt-3 space-y-2">
            {networks.length === 0 ? (
              <p className="text-sm text-slate-300">No networks registered yet.</p>
            ) : (
              networks.map((network) => (
                <div
                  key={network.network_id}
                  className="rounded-xl border border-cyan-900/40 bg-slate-950/50 p-3"
                >
                  <div className="flex items-center justify-between gap-2">
                    <p className="font-semibold">{network.name}</p>
                    <p className="text-xs text-cyan-200">{network.status}</p>
                  </div>
                  <p className="font-mono text-xs text-slate-400">{network.network_id}</p>
                </div>
              ))
            )}
          </div>
        </article>

        <article className="glass-card rounded-2xl p-5">
          <h3 className="section-title">Recent Job Queue</h3>
          <div className="mt-3 space-y-2">
            {jobs.length === 0 ? (
              <p className="text-sm text-slate-300">No jobs submitted yet.</p>
            ) : (
              jobs.slice(0, 8).map((job) => (
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
    </main>
  );
}
