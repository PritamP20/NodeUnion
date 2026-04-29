"use client";

import { useEffect, useMemo, useState } from "react";
import useSWR from "swr";
import { motion } from "framer-motion";
import { useRouter, useSearchParams } from "next/navigation";
import {
  ChevronDown,
  ChevronRight,
  Globe,
  Rocket,
  Server,
  TerminalSquare,
  Timer,
  Wallet,
} from "lucide-react";
import { JobStatusBadge } from "@/components/job-status-badge";
import { LiveBadge } from "@/components/live-badge";
import { MetricCard } from "@/components/metric-card";
import { StatusDot } from "@/components/status-dot";
import { fetchMainSnapshot, statusLabel, submitJob } from "@/lib/orchestrator-realtime";

type Snapshot = Awaited<ReturnType<typeof fetchMainSnapshot>>;

type DeploymentRecord = {
  jobId: string;
  status: string;
  message: string;
  deployUrl?: string | null;
  createdAt: string;
};

function parseCommand(input: string): string[] | undefined {
  const trimmed = input.trim();
  if (!trimmed) return undefined;
  return trimmed.split(/\s+/).filter(Boolean);
}

function formatDuration(createdAtEpoch: number) {
  const elapsedMins = Math.max(1, Math.round((Date.now() / 1000 - createdAtEpoch) / 60));
  if (elapsedMins < 60) return `${elapsedMins}m`;
  const hours = Math.floor(elapsedMins / 60);
  const mins = elapsedMins % 60;
  return `${hours}h ${mins}m`;
}

function mapJobStatus(status: Snapshot["jobs"][number]["status"]) {
  return statusLabel(status) as "Running" | "Queued" | "Completed" | "Failed";
}

function jobStatusTone(status: string) {
  if (status === "Running") return "online";
  if (status === "Queued") return "degraded";
  if (status === "Completed") return "online";
  return "offline";
}

export function ProviderPageClient() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { data, error, isLoading, mutate } = useSWR<Snapshot>("/api/main/snapshot", () => fetchMainSnapshot(), {
    refreshInterval: 30000,
    revalidateOnFocus: true,
  });

  const [wallet, setWallet] = useState("");
  const [image, setImage] = useState("ghcr.io/nodeunion/demo:latest");
  const [command, setCommand] = useState("python app.py");
  const [cpu, setCpu] = useState(2);
  const [ram, setRam] = useState(4096);
  const [exposedPort, setExposedPort] = useState(3000);
  const [networkId, setNetworkId] = useState("");
  const [orchestratorUrl, setOrchestratorUrl] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [deployments, setDeployments] = useState<DeploymentRecord[]>([]);
  const [selectedJobId, setSelectedJobId] = useState<string>("");

  const liveNetworks = data?.networks ?? [];
  const liveJobs = useMemo(() => {
    return [...(data?.jobs ?? [])].sort((left, right) => right.created_at_epoch_secs - left.created_at_epoch_secs).slice(0, 14);
  }, [data?.jobs]);

  const selectedNetwork = networkId || liveNetworks[0]?.network_id || "";
  const selectedNetworkRecord = liveNetworks.find((network) => network.network_id === selectedNetwork);
  const selectedJob = liveJobs.find((job) => job.job_id === selectedJobId);

  const deploymentFeed = useMemo(() => deployments.slice(0, 5), [deployments]);
  const activeNodes = data?.nodes.filter((node) => node.status !== "Offline").length ?? 0;
  const activeJobs = liveJobs.filter((job) => ["Pending", "Scheduled", "Running"].includes(job.status)).length;
  const liveNetworksCount = liveNetworks.length;

  useEffect(() => {
    const walletValue = searchParams.get("wallet");
    const networkValue = searchParams.get("network");
    const imageValue = searchParams.get("image");
    const commandValue = searchParams.get("command");
    const cpuValue = searchParams.get("cpu");
    const ramGbValue = searchParams.get("ramGb");
    const portValue = searchParams.get("port");
    const orchestratorUrlValue = searchParams.get("orchestratorUrl");

    if (walletValue) setWallet(walletValue);
    if (networkValue) setNetworkId(networkValue);
    if (imageValue) setImage(imageValue);
    if (commandValue) setCommand(commandValue);
    if (cpuValue) {
      const parsedCpu = Number(cpuValue);
      if (Number.isFinite(parsedCpu) && parsedCpu > 0) {
        setCpu(parsedCpu);
      }
    }

    if (ramGbValue) {
      const parsedRamGb = Number(ramGbValue);
      if (Number.isFinite(parsedRamGb) && parsedRamGb > 0) {
        setRam(Math.round(parsedRamGb * 1024));
      }
    }

    if (portValue) {
      const parsedPort = Number(portValue);
      if (Number.isFinite(parsedPort) && parsedPort > 0) {
        setExposedPort(parsedPort);
      }
    }

    if (orchestratorUrlValue) {
      setOrchestratorUrl(orchestratorUrlValue);
    }
  }, [searchParams]);

  const onSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setSubmitError(null);

    const selectedNetworkId = networkId || liveNetworks[0]?.network_id || "";

    if (!wallet.trim() || !selectedNetworkId || !image.trim()) {
      setSubmitError("wallet, network, and image are required");
      return;
    }

    if (!Number.isFinite(cpu) || cpu < 0.5 || cpu > 16) {
      setSubmitError("cpu limit must be between 0.5 and 16 cores");
      return;
    }

    if (!Number.isFinite(ram) || ram < 512 || ram > 65536) {
      setSubmitError("ram limit must be between 512 MB and 65536 MB");
      return;
    }

    if (!Number.isFinite(exposedPort) || exposedPort < 1 || exposedPort > 65535) {
      setSubmitError("exposed port must be between 1 and 65535");
      return;
    }

    setSubmitting(true);

    try {
      const response = await submitJob({
        network_id: selectedNetworkId,
        user_wallet: wallet.trim(),
        image: image.trim(),
        cpu_limit: cpu,
        ram_limit_mb: ram,
        command: parseCommand(command),
        exposed_port: exposedPort,
        orchestrator_url: orchestratorUrl.trim() || undefined,
      });

      setDeployments((current) => [
        {
          jobId: response.job_id,
          status: response.status,
          message: response.message,
          deployUrl: response.deploy_url,
          createdAt: new Date().toLocaleString(),
        },
        ...current,
      ]);
      setSelectedJobId(response.job_id);
      await mutate();
    } catch (submitError) {
      setSubmitError(submitError instanceof Error ? submitError.message : "failed to submit deployment");
    } finally {
      setSubmitting(false);
    }
  };

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
          <span>Deploy console</span>
          <span>{error ? "snapshot error" : "snapshot live"}</span>
        </div>

        <div className="mt-4 flex flex-wrap items-end justify-between gap-4">
          <div>
            <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">Launch workloads with the web dashboard.</h1>
            <p className="mt-3 max-w-3xl text-sm leading-7 text-slate-300 sm:text-base">
              Submit container jobs, choose a live network, and track recent deployments without leaving the dashboard.
            </p>
          </div>

          <div className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-slate-300">
            <p className="text-[11px] uppercase tracking-[0.28em] text-slate-500">Selected network</p>
            <p className="mt-2 font-mono text-sm text-slate-100">{selectedNetworkRecord?.name ?? "choose a network"}</p>
          </div>
        </div>

        <div className="mt-6 grid gap-3 sm:grid-cols-3">
          <MetricCard label="Live Networks" value={isLoading ? "—" : liveNetworksCount.toLocaleString()} icon={<Globe size={16} className="text-cyan-300" />} />
          <MetricCard label="Active Nodes" value={isLoading ? "—" : activeNodes.toLocaleString()} icon={<Server size={16} className="text-indigo-300" />} />
          <MetricCard label="Active Jobs" value={isLoading ? "—" : activeJobs.toLocaleString()} icon={<Rocket size={16} className="text-cyan-300" />} />
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
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <Rocket size={15} className="text-cyan-300" />
            Job submission
          </div>

          <form onSubmit={onSubmit} className="mt-5 grid gap-4">
            <label className="text-sm text-slate-300">
              Wallet address
              <input
                value={wallet}
                onChange={(event) => setWallet(event.target.value)}
                className="mt-2 w-full rounded-2xl px-4 py-3 font-mono text-sm"
                placeholder="solana wallet address"
              />
            </label>

            <label className="text-sm text-slate-300">
              Network selector
              <select
                value={networkId || liveNetworks[0]?.network_id || ""}
                onChange={(event) => setNetworkId(event.target.value)}
                className="mt-2 w-full rounded-2xl px-4 py-3 font-mono text-sm"
              >
                <option value="" disabled>
                  Select a network
                </option>
                {liveNetworks.map((network) => (
                  <option key={network.network_id} value={network.network_id}>
                    {network.name} ({network.network_id})
                  </option>
                ))}
              </select>
            </label>

            <label className="text-sm text-slate-300">
              Container image
              <input
                value={image}
                onChange={(event) => setImage(event.target.value)}
                className="mt-2 w-full rounded-2xl px-4 py-3 font-mono text-sm"
                placeholder="ghcr.io/example/app:latest"
              />
            </label>

            <label className="text-sm text-slate-300">
              Command
              <input
                value={command}
                onChange={(event) => setCommand(event.target.value)}
                className="mt-2 w-full rounded-2xl px-4 py-3 font-mono text-sm"
                placeholder="python app.py"
              />
            </label>

            <div className="grid gap-4 md:grid-cols-2">
              <label className="text-sm text-slate-300">
                CPU limit: <span className="font-mono text-slate-100">{cpu.toFixed(1)}</span>
                <input
                  type="range"
                  min="0.5"
                  max="16"
                  step="0.5"
                  value={cpu}
                  onChange={(event) => setCpu(Number(event.target.value))}
                  className="mt-3 w-full accent-indigo-500"
                />
              </label>

              <label className="text-sm text-slate-300">
                RAM limit: <span className="font-mono text-slate-100">{ram} MB</span>
                <input
                  type="range"
                  min="512"
                  max="65536"
                  step="512"
                  value={ram}
                  onChange={(event) => setRam(Number(event.target.value))}
                  className="mt-3 w-full accent-cyan-400"
                />
              </label>
            </div>

            <label className="text-sm text-slate-300">
              Exposed port
              <input
                type="number"
                min="1"
                max="65535"
                value={exposedPort}
                onChange={(event) => setExposedPort(Number(event.target.value))}
                className="mt-2 w-full rounded-2xl px-4 py-3 font-mono text-sm"
              />
            </label>

            <label className="text-sm text-slate-300">
              Orchestrator URL override
              <input
                value={orchestratorUrl}
                onChange={(event) => setOrchestratorUrl(event.target.value)}
                className="mt-2 w-full rounded-2xl px-4 py-3 font-mono text-sm"
                placeholder="https://..."
              />
            </label>

            <button
              type="submit"
              disabled={submitting}
              className="inline-flex items-center justify-center gap-2 rounded-full bg-indigo-500 px-5 py-3 text-sm font-semibold text-white transition hover:bg-indigo-400 disabled:cursor-not-allowed disabled:opacity-60"
            >
              {submitting ? "Submitting..." : "Submit Job"}
            </button>

            {submitError ? (
              <p className="rounded-2xl border border-rose-500/20 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
                {submitError}
              </p>
            ) : null}
          </form>
        </motion.article>

        <motion.article
          initial={{ opacity: 0, y: 12 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, amount: 0.2 }}
          transition={{ duration: 0.35, ease: "easeOut" }}
          className="glass-card rounded-[2rem] p-6"
        >
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <TerminalSquare size={15} className="text-cyan-300" />
            Live deployment feed
          </div>

          <div className="mt-5 space-y-4">
            {deploymentFeed.length === 0 ? (
              <div className="rounded-2xl border border-white/10 bg-white/5 p-4 text-sm text-slate-400">
                Submitted jobs will appear here immediately after deployment.
              </div>
            ) : (
              deploymentFeed.map((deployment) => (
                <div key={`${deployment.jobId}-${deployment.createdAt}`} className="rounded-2xl border border-white/10 bg-white/5 p-4">
                  <div className="flex items-center justify-between gap-3">
                    <div>
                      <p className="font-mono text-xs text-slate-500">{deployment.createdAt}</p>
                      <p className="mt-1 font-mono text-sm text-slate-100">{deployment.jobId}</p>
                    </div>
                    <JobStatusBadge state={deployment.status as Parameters<typeof JobStatusBadge>[0]["state"]} />
                  </div>
                  <p className="mt-3 text-sm leading-6 text-slate-300">{deployment.message}</p>
                  {deployment.deployUrl ? (
                    <a
                      href={deployment.deployUrl}
                      target="_blank"
                      rel="noreferrer"
                      className="mt-3 inline-flex text-sm text-cyan-300 underline"
                    >
                      Open deploy URL
                    </a>
                  ) : null}
                </div>
              ))
            )}
          </div>

          <div className="mt-6 overflow-hidden rounded-2xl border border-white/10">
            <table className="min-w-full text-left text-sm">
              <thead className="bg-white/5 text-slate-400">
                <tr>
                  <th className="px-4 py-3 font-medium">Job</th>
                  <th className="px-4 py-3 font-medium">Status</th>
                  <th className="px-4 py-3 font-medium">Node</th>
                  <th className="px-4 py-3 font-medium">Started</th>
                  <th className="px-4 py-3 font-medium">Duration</th>
                </tr>
              </thead>
              <tbody>
                {liveJobs.length === 0 ? (
                  <tr>
                    <td className="px-4 py-6 text-slate-400" colSpan={5}>
                      No live jobs available yet.
                    </td>
                  </tr>
                ) : (
                  liveJobs.map((job) => {
                    const isSelected = selectedJobId === job.job_id;

                    return (
                      <tr
                        key={job.job_id}
                        onClick={() => {
                          setSelectedJobId(job.job_id);
                          router.push(`/jobs/${encodeURIComponent(job.job_id)}`);
                        }}
                        className={`cursor-pointer border-t border-white/10 transition ${isSelected ? "bg-indigo-500/10" : "hover:bg-white/5"}`}
                      >
                        <td className="px-4 py-4 font-mono text-xs text-slate-100">{job.job_id}</td>
                        <td className="px-4 py-4">
                          <span className="inline-flex items-center gap-2">
                            <StatusDot tone={jobStatusTone(mapJobStatus(job.status))} />
                            <JobStatusBadge state={mapJobStatus(job.status)} />
                          </span>
                        </td>
                        <td className="px-4 py-4 text-slate-300">{job.assigned_node_id || "unassigned"}</td>
                        <td className="px-4 py-4 text-slate-300">{formatDuration(job.created_at_epoch_secs)}</td>
                        <td className="px-4 py-4 text-slate-300">
                          {job.status === "Running" ? "live" : job.status === "Done" ? "settled" : "queued"}
                        </td>
                      </tr>
                    );
                  })
                )}
              </tbody>
            </table>
          </div>

          <div className="mt-5 rounded-2xl border border-white/10 bg-white/5 p-5">
            <button
              type="button"
              onClick={() => setSelectedJobId((current) => current || liveJobs[0]?.job_id || "")}
              className="flex w-full items-center justify-between gap-3 text-left"
            >
              <div>
                <p className="text-[11px] uppercase tracking-[0.28em] text-slate-500">Expanded job details</p>
                <p className="mt-2 font-semibold text-slate-100">
                  {selectedJob ? selectedJob.job_id : "Select a job from the table"}
                </p>
              </div>
              {selectedJob ? <ChevronDown size={16} className="text-slate-400" /> : <ChevronRight size={16} className="text-slate-400" />}
            </button>

            {selectedJob ? (
              <div className="mt-4 grid gap-4 sm:grid-cols-2">
                <div className="rounded-2xl border border-white/10 bg-[#0b1018] p-4 font-mono text-sm leading-6 text-slate-200">
                  <p className="text-[11px] uppercase tracking-[0.24em] text-slate-500">Execution details</p>
                  <p className="mt-3">image: {selectedJob.image}</p>
                  <p>network: {selectedJob.network_id}</p>
                  <p>status: {selectedJob.status}</p>
                  <p>node: {selectedJob.assigned_node_id || "unassigned"}</p>
                  <p>cpu: {selectedJob.cpu_limit.toFixed(2)} cores</p>
                  <p>ram: {Math.round(selectedJob.ram_limit_mb / 1024)} GB</p>
                </div>

                <div className="rounded-2xl border border-white/10 bg-[#0b1018] p-4 font-mono text-sm leading-6 text-slate-200">
                  <p className="text-[11px] uppercase tracking-[0.24em] text-slate-500">Event trail</p>
                  <div className="mt-3 space-y-2 text-slate-300">
                    <p>&gt; submitted to /api/main/submit-job</p>
                    <p>&gt; accepted by orchestrator</p>
                    <p>&gt; status mapped to {mapJobStatus(selectedJob.status)}</p>
                    <p>&gt; assigned node {selectedJob.assigned_node_id || "pending"}</p>
                    <p>&gt; duration {formatDuration(selectedJob.created_at_epoch_secs)}</p>
                  </div>
                </div>
              </div>
            ) : null}
          </div>

          <div className="mt-5 rounded-2xl border border-cyan-500/20 bg-cyan-500/10 p-4 text-sm leading-6 text-cyan-100">
            The current API does not expose raw container logs yet, so the expanded view focuses on live job metadata and the execution trail already available from the orchestrator.
          </div>
        </motion.article>
      </section>

      <section className="mt-6 grid gap-6 xl:grid-cols-2">
        <div className="glass-card rounded-[2rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <Timer size={15} className="text-cyan-300" />
            Notes
          </div>
          <ul className="mt-4 space-y-2 text-sm leading-6 text-slate-300">
            <li>Expose the app port to get an automatic public URL when the provider agent supports it.</li>
            <li>Override the orchestrator URL only when a job needs to target a specific control plane.</li>
            <li>The deployment feed updates immediately after submit and the live snapshot refreshes automatically.</li>
          </ul>
        </div>

        <div className="glass-card rounded-[2rem] p-6">
          <div className="flex items-center gap-2 text-sm uppercase tracking-[0.28em] text-slate-400">
            <Wallet size={15} className="text-indigo-300" />
            Submission status
          </div>
          <p className="mt-4 text-sm leading-6 text-slate-300">
            Use the form on the left to launch a job, then watch the deployment feed and job table update as the orchestrator accepts the request.
          </p>
        </div>
      </section>
    </main>
  );
}
