const ORCHESTRATOR_PROXY_BASE = "/api/orchestrator";
const MAIN_API_BASE = "/api/main";

export type OrchestratorNetwork = {
  network_id: string;
  name: string;
  description?: string | null;
  status: string;
  orchestrator_url?: string | null;
  created_at_epoch_secs: number;
};

export type OrchestratorNode = {
  node_id: string;
  network_id: string;
  agent_url?: string | null;
  provider_wallet?: string | null;
  region?: string | null;
  status: "Idle" | "Busy" | "Draining" | "Preempting" | "Offline";
  cpu_available_pct: number;
  ram_available_mb: number;
  running_chunks: number;
  last_seen_epoch_secs: number;
  orchestrator_url?: string;
};

export type OrchestratorJob = {
  job_id: string;
  network_id: string;
  user_wallet?: string | null;
  image: string;
  cpu_limit: number;
  ram_limit_mb: number;
  status: "Pending" | "Scheduled" | "Running" | "Done" | "Failed" | "Preempted" | "Stopped";
  assigned_node_id?: string | null;
  created_at_epoch_secs: number;
  orchestrator_url?: string;
};

export type OrchestratorEntitlement = {
  entitlement_id: string;
  user_wallet: string;
  network_id: string;
  bought_units: number;
  used_units: number;
  created_at_epoch_secs: number;
  orchestrator_url?: string;
};

export type OrchestratorSettlement = {
  settlement_id: string;
  job_id: string;
  user_wallet: string;
  provider_wallet?: string | null;
  network_id: string;
  amount_tokens: number;
  tx_status?: string | null;
  created_at_epoch_secs: number;
  orchestrator_url?: string;
};

export type AggregatedSnapshot = {
  sources: Array<{
    orchestrator_url: string;
    ok: boolean;
    error?: string;
  }>;
  networks: OrchestratorNetwork[];
  nodes: OrchestratorNode[];
  jobs: OrchestratorJob[];
  entitlements: OrchestratorEntitlement[];
  settlements: OrchestratorSettlement[];
  has_live_data: boolean;
};

async function fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${ORCHESTRATOR_PROXY_BASE}${path}`, {
    ...init,
    cache: "no-store",
    headers: {
      "content-type": "application/json",
      ...(init?.headers ?? {}),
    },
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `request failed with ${response.status}`);
  }

  return (await response.json()) as T;
}

export async function fetchNetworks() {
  return fetchJson<OrchestratorNetwork[]>("/networks");
}

export async function fetchNodes() {
  return fetchJson<OrchestratorNode[]>("/nodes");
}

export async function fetchJobs() {
  return fetchJson<OrchestratorJob[]>("/jobs");
}

export async function fetchWalletEntitlements(wallet: string) {
  return fetchJson<OrchestratorEntitlement[]>(`/users/${encodeURIComponent(wallet)}/entitlements`);
}

export async function fetchWalletSettlements(wallet: string) {
  return fetchJson<OrchestratorSettlement[]>(`/users/${encodeURIComponent(wallet)}/settlements`);
}

export async function submitJob(payload: {
  network_id: string;
  user_wallet: string;
  image: string;
  cpu_limit: number;
  ram_limit_mb: number;
  command?: string[];
  exposed_port?: number;
  orchestrator_url?: string;
}) {
  const response = await fetch(`${MAIN_API_BASE}/submit-job`, {
    method: "POST",
    cache: "no-store",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify(payload),
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `request failed with ${response.status}`);
  }

  return (await response.json()) as {
    accepted: boolean;
    job_id: string;
    status: string;
    deploy_url?: string | null;
    message: string;
  };
}

export async function fetchMainSnapshot(wallet?: string) {
  const query = wallet ? `?wallet=${encodeURIComponent(wallet)}` : "";
  const response = await fetch(`${MAIN_API_BASE}/snapshot${query}`, {
    cache: "no-store",
    headers: {
      "content-type": "application/json",
    },
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `request failed with ${response.status}`);
  }

  return (await response.json()) as AggregatedSnapshot;
}

export function formatRelativeTime(epochSecs: number) {
  const delta = Math.max(0, Math.floor(Date.now() / 1000) - epochSecs);
  if (delta < 60) return `${delta}s ago`;
  if (delta < 3600) return `${Math.floor(delta / 60)}m ago`;
  if (delta < 86400) return `${Math.floor(delta / 3600)}h ago`;
  return `${Math.floor(delta / 86400)}d ago`;
}

export function estimateJobCredits(job: Pick<OrchestratorJob, "cpu_limit" | "ram_limit_mb">) {
  return Math.round(job.cpu_limit * 100 + job.ram_limit_mb / 256);
}

export function statusLabel(status: OrchestratorJob["status"]) {
  if (status === "Pending") return "Queued";
  if (status === "Scheduled") return "Running";
  if (status === "Done") return "Completed";
  if (status === "Stopped") return "Failed";
  return status;
}

export function clampMin(value: number, min = 0) {
  return Number.isFinite(value) ? Math.max(min, value) : min;
}
