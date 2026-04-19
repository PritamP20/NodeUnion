"use client";

import { useEffect, useMemo, useState } from "react";
import { Globe, Rocket, Server, TerminalSquare } from "lucide-react";
import { fetchMainSnapshot, submitJob, type OrchestratorJob, type OrchestratorNetwork } from "@/lib/orchestrator-realtime";

type DeploymentRecord = {
  jobId: string;
  networkId: string;
  status: string;
  deployUrl?: string | null;
  message: string;
  createdAt: string;
};

function parseCommand(input: string): string[] | undefined {
  const trimmed = input.trim();
  if (!trimmed) return undefined;
  return trimmed.split(/\s+/).filter(Boolean);
}

export default function ProviderPage() {
  const [wallet, setWallet] = useState("");
  const [image, setImage] = useState("nginx:alpine");
  const [command, setCommand] = useState("");
  const [cpu, setCpu] = useState("0.5");
  const [ram, setRam] = useState("512");
  const [exposedPort, setExposedPort] = useState("80");
  const [networkId, setNetworkId] = useState("");
  const [orchestratorUrl, setOrchestratorUrl] = useState("");

  const [networks, setNetworks] = useState<OrchestratorNetwork[]>([]);
  const [jobs, setJobs] = useState<OrchestratorJob[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [deployments, setDeployments] = useState<DeploymentRecord[]>([]);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      try {
        const snapshot = await fetchMainSnapshot();
        if (cancelled) return;

        setNetworks(snapshot.networks);
        setJobs(snapshot.jobs);
        if (!networkId && snapshot.networks[0]?.network_id) {
          setNetworkId(snapshot.networks[0].network_id);
          setOrchestratorUrl(snapshot.networks[0].orchestrator_url ?? "");
        }
      } catch (err) {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : "failed to load orchestrator snapshot");
      }
    };

    void load();
    const timer = window.setInterval(() => {
      void load();
    }, 8000);

    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [networkId]);

  const recentJobs = useMemo(() => {
    return [...jobs]
      .sort((a, b) => b.created_at_epoch_secs - a.created_at_epoch_secs)
      .slice(0, 12);
  }, [jobs]);

  const onSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError(null);

    if (!wallet.trim() || !networkId.trim() || !image.trim()) {
      setError("wallet, network, and image are required");
      return;
    }

    const cpuLimit = Number.parseFloat(cpu);
    const ramLimit = Number.parseInt(ram, 10);
    const portValue = Number.parseInt(exposedPort, 10);

    if (!Number.isFinite(cpuLimit) || cpuLimit <= 0) {
      setError("cpu limit must be a positive number");
      return;
    }

    if (!Number.isFinite(ramLimit) || ramLimit <= 0) {
      setError("ram limit must be a positive integer");
      return;
    }

    if (!Number.isFinite(portValue) || portValue <= 0 || portValue > 65535) {
      setError("exposed port must be between 1 and 65535");
      return;
    }

    setSubmitting(true);

    try {
      const response = await submitJob({
        network_id: networkId,
        user_wallet: wallet.trim(),
        image: image.trim(),
        cpu_limit: cpuLimit,
        ram_limit_mb: ramLimit,
        command: parseCommand(command),
        exposed_port: portValue,
        orchestrator_url: orchestratorUrl.trim() || undefined,
      });

      setDeployments((current) => [
        {
          jobId: response.job_id,
          networkId,
          status: response.status,
          deployUrl: response.deploy_url,
          message: response.message,
          createdAt: new Date().toLocaleString(),
        },
        ...current,
      ]);
    } catch (err) {
      setError(err instanceof Error ? err.message : "failed to submit deployment");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main className="mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
      <section className="glass-card rounded-[2rem] p-6 sm:p-8">
        <div className="flex flex-wrap items-center gap-3 text-[11px] uppercase tracking-[0.3em] text-slate-400">
          <span className="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-emerald-300">
            Deploy Console
          </span>
          <span>Launch projects from dashboard</span>
        </div>

        <h1 className="mt-4 text-4xl font-semibold tracking-tight sm:text-5xl">
          Deploy your project and get a public URL.
        </h1>
        <p className="mt-3 max-w-3xl text-sm leading-6 text-slate-300">
          Submit a workload directly to NodeUnion. If the app exposes a port, the agent auto-creates an ngrok tunnel and returns a deploy URL.
        </p>
      </section>

      <section className="mt-6 grid gap-6 xl:grid-cols-[1fr_1fr]">
        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="mb-4 flex items-center gap-2 text-sm uppercase tracking-[0.24em] text-slate-400">
            <Rocket size={16} className="text-sky-300" />
            Deploy Project
          </div>

          <form onSubmit={onSubmit} className="grid gap-4">
            <label className="text-sm text-slate-300">
              Wallet
              <input value={wallet} onChange={(e) => setWallet(e.target.value)} className="mt-1 w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2" placeholder="solana wallet address" />
            </label>

            <label className="text-sm text-slate-300">
              Network
              <select value={networkId} onChange={(e) => setNetworkId(e.target.value)} className="mt-1 w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2">
                {networks.map((network) => (
                  <option key={`${network.orchestrator_url || "local"}-${network.network_id}`} value={network.network_id}>
                    {network.network_id} ({network.name})
                  </option>
                ))}
              </select>
            </label>

            <label className="text-sm text-slate-300">
              Image
              <input value={image} onChange={(e) => setImage(e.target.value)} className="mt-1 w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2" placeholder="nginx:alpine" />
            </label>

            <label className="text-sm text-slate-300">
              Command (optional)
              <input value={command} onChange={(e) => setCommand(e.target.value)} className="mt-1 w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2" placeholder="sh -c 'echo hello'" />
            </label>

            <div className="grid grid-cols-3 gap-3">
              <label className="text-sm text-slate-300">
                CPU
                <input value={cpu} onChange={(e) => setCpu(e.target.value)} className="mt-1 w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2" />
              </label>
              <label className="text-sm text-slate-300">
                RAM MB
                <input value={ram} onChange={(e) => setRam(e.target.value)} className="mt-1 w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2" />
              </label>
              <label className="text-sm text-slate-300">
                Exposed port
                <input value={exposedPort} onChange={(e) => setExposedPort(e.target.value)} className="mt-1 w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2" />
              </label>
            </div>

            <label className="text-sm text-slate-300">
              Orchestrator URL (optional override)
              <input value={orchestratorUrl} onChange={(e) => setOrchestratorUrl(e.target.value)} className="mt-1 w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2" placeholder="https://..." />
            </label>

            <button type="submit" disabled={submitting} className="rounded-xl bg-sky-500 px-4 py-3 text-sm font-semibold text-white disabled:opacity-60">
              {submitting ? "Deploying..." : "Deploy"}
            </button>

            {error ? <p className="rounded-lg border border-rose-500/30 bg-rose-500/10 px-3 py-2 text-sm text-rose-300">{error}</p> : null}
          </form>
        </article>

        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="mb-4 flex items-center gap-2 text-sm uppercase tracking-[0.24em] text-slate-400">
            <Globe size={16} className="text-emerald-300" />
            Deployments
          </div>

          <div className="space-y-3">
            {deployments.length === 0 ? (
              <div className="rounded-xl border border-white/10 bg-black/20 p-4 text-sm text-slate-300">
                No deployments submitted from this dashboard session yet.
              </div>
            ) : (
              deployments.map((deployment) => (
                <div key={`${deployment.jobId}-${deployment.createdAt}`} className="rounded-xl border border-white/10 bg-black/20 p-4">
                  <p className="font-mono text-xs text-slate-400">{deployment.createdAt}</p>
                  <p className="mt-1 text-sm text-slate-200">Job: {deployment.jobId}</p>
                  <p className="text-sm text-slate-300">Status: {deployment.status}</p>
                  <p className="text-sm text-slate-400">{deployment.message}</p>
                  {deployment.deployUrl ? (
                    <a href={deployment.deployUrl} target="_blank" rel="noreferrer" className="mt-2 inline-flex text-sm text-emerald-300 underline">
                      Open deploy URL
                    </a>
                  ) : (
                    <p className="mt-2 text-sm text-amber-300">Deploy URL not available yet.</p>
                  )}
                </div>
              ))
            )}
          </div>
        </article>
      </section>

      <section className="mt-6 grid gap-6 xl:grid-cols-[1fr_1fr]">
        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="mb-4 flex items-center gap-2 text-sm uppercase tracking-[0.24em] text-slate-400">
            <Server size={16} className="text-sky-300" />
            Recent Jobs
          </div>
          <div className="space-y-2">
            {recentJobs.length === 0 ? (
              <p className="text-sm text-slate-400">No recent jobs.</p>
            ) : (
              recentJobs.map((job) => (
                <div key={job.job_id} className="rounded-lg border border-white/10 bg-black/20 px-3 py-2 text-sm">
                  <p className="font-mono text-xs text-slate-400">{job.job_id}</p>
                  <p className="text-slate-200">{job.image}</p>
                  <p className="text-slate-400">{job.status} {job.assigned_node_id ? `• node ${job.assigned_node_id}` : ""}</p>
                </div>
              ))
            )}
          </div>
        </article>

        <article className="glass-card rounded-[1.75rem] p-6">
          <div className="mb-4 flex items-center gap-2 text-sm uppercase tracking-[0.24em] text-slate-400">
            <TerminalSquare size={16} className="text-sky-300" />
            Notes
          </div>
          <ul className="space-y-2 text-sm text-slate-300">
            <li>Expose the app port to get an automatic ngrok public URL.</li>
            <li>Set AUTO_NGROK_EXPOSE=true on provider agents for auto tunnel creation.</li>
            <li>If ngrok is missing or unauthenticated, deployment still runs but URL may be unavailable.</li>
          </ul>
        </article>
      </section>
    </main>
  );
}
