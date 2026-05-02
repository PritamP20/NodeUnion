import { NextRequest, NextResponse } from "next/server";
import { discoverOrchestratorUrls } from "@/lib/orchestrator-discovery";

export const runtime = "nodejs";

type NetworkRecord = {
  network_id: string;
  name: string;
  description?: string | null;
  status: string;
  orchestrator_url?: string | null;
  created_at_epoch_secs: number;
};

type NodeRecord = {
  node_id: string;
  network_id: string;
  agent_url?: string | null;
  provider_wallet?: string | null;
  region?: string | null;
  status: string;
  cpu_available_pct: number;
  ram_available_mb: number;
  running_chunks: number;
  last_seen_epoch_secs: number;
  orchestrator_url?: string;
};

type JobRecord = {
  job_id: string;
  network_id: string;
  user_wallet?: string | null;
  image: string;
  cpu_limit: number;
  ram_limit_mb: number;
  status: string;
  assigned_node_id?: string | null;
  created_at_epoch_secs: number;
  orchestrator_url?: string;
};

type EntitlementRecord = {
  entitlement_id: string;
  user_wallet: string;
  network_id: string;
  bought_units: number;
  used_units: number;
  created_at_epoch_secs: number;
  orchestrator_url?: string;
};

type SettlementRecord = {
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

type SourceStatus = {
  orchestrator_url: string;
  ok: boolean;
  networks_ok: boolean;
  nodes_ok: boolean;
  jobs_ok: boolean;
  wallet_ok: boolean;
  error?: string;
};

async function fetchOrchestratorJson<T>(baseUrl: string, path: string): Promise<T> {
  const response = await fetch(`${baseUrl}${path}`, { cache: "no-store" });
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `failed ${path} with ${response.status}`);
  }
  return (await response.json()) as T;
}

async function fetchFromSource(baseUrl: string, wallet?: string | null) {
  const source: SourceStatus = {
    orchestrator_url: baseUrl,
    ok: false,
    networks_ok: false,
    nodes_ok: false,
    jobs_ok: false,
    wallet_ok: !wallet,
  };

  let networks: NetworkRecord[] = [];
  let nodes: NodeRecord[] = [];
  let jobs: JobRecord[] = [];
  let entitlements: EntitlementRecord[] = [];
  let settlements: SettlementRecord[] = [];

  const endpointErrors: string[] = [];

  try {
    networks = await fetchOrchestratorJson<NetworkRecord[]>(baseUrl, "/networks");
    source.networks_ok = true;
  } catch (error) {
    endpointErrors.push(
      `networks: ${error instanceof Error ? error.message : "request failed"}`,
    );
  }

  try {
    nodes = await fetchOrchestratorJson<NodeRecord[]>(baseUrl, "/nodes");
    source.nodes_ok = true;
  } catch (error) {
    endpointErrors.push(`nodes: ${error instanceof Error ? error.message : "request failed"}`);
  }

  try {
    jobs = await fetchOrchestratorJson<JobRecord[]>(baseUrl, "/jobs");
    source.jobs_ok = true;
  } catch (error) {
    endpointErrors.push(`jobs: ${error instanceof Error ? error.message : "request failed"}`);
  }

  if (wallet) {
    try {
      [entitlements, settlements] = await Promise.all([
        fetchOrchestratorJson<EntitlementRecord[]>(
          baseUrl,
          `/users/${encodeURIComponent(wallet)}/entitlements`,
        ),
        fetchOrchestratorJson<SettlementRecord[]>(
          baseUrl,
          `/users/${encodeURIComponent(wallet)}/settlements`,
        ),
      ]);
      source.wallet_ok = true;
    } catch (error) {
      endpointErrors.push(`wallet: ${error instanceof Error ? error.message : "request failed"}`);
      // Wallet APIs are optional; keep partial source data.
    }
  }

  source.ok = source.networks_ok || source.nodes_ok || source.jobs_ok;
  if (endpointErrors.length > 0) {
    source.error = endpointErrors.join(" | ");
  }

  return {
    source,
    networks: networks.map((item) => ({
      ...item,
      orchestrator_url: item.orchestrator_url || baseUrl,
    })),
    nodes: nodes.map((item) => ({ ...item, agent_url: item.agent_url ?? null, orchestrator_url: baseUrl })),
    jobs: jobs.map((item) => ({ ...item, orchestrator_url: baseUrl })),
    entitlements: entitlements.map((item) => ({ ...item, orchestrator_url: baseUrl })),
    settlements: settlements.map((item) => ({ ...item, orchestrator_url: baseUrl })),
  };
}

export async function GET(req: NextRequest) {
  const wallet = req.nextUrl.searchParams.get("wallet");
  const orchestratorUrls = await discoverOrchestratorUrls();

  const chunks = await Promise.all(orchestratorUrls.map((url) => fetchFromSource(url, wallet)));

  const sources = chunks.map((chunk) => chunk.source);
  const networks = chunks.flatMap((chunk) => chunk.networks);
  const nodes = chunks.flatMap((chunk) => chunk.nodes);
  const jobs = chunks.flatMap((chunk) => chunk.jobs);
  const entitlements = chunks.flatMap((chunk) => chunk.entitlements);
  const settlements = chunks.flatMap((chunk) => chunk.settlements);

  return NextResponse.json(
    {
      sources,
      networks,
      nodes,
      jobs,
      entitlements,
      settlements,
      has_live_data: sources.some((source) => source.ok),
    },
    { status: sources.some((source) => source.ok) ? 200 : 503 },
  );
}
