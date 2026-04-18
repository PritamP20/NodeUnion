import { NextRequest, NextResponse } from "next/server";
import { discoverOrchestratorUrls } from "@/lib/orchestrator-discovery";

export const runtime = "nodejs";

type SubmitPayload = {
  network_id: string;
  user_wallet: string;
  image: string;
  cpu_limit: number;
  ram_limit_mb: number;
  command?: string[];
  orchestrator_url?: string;
};

type NetworkRecord = {
  network_id: string;
  orchestrator_url?: string | null;
};

async function fetchNetworks(baseUrl: string) {
  const response = await fetch(`${baseUrl}/networks`, { cache: "no-store" });
  if (!response.ok) {
    return [] as NetworkRecord[];
  }
  return (await response.json()) as NetworkRecord[];
}

async function resolveTargetOrchestrator(payload: SubmitPayload) {
  if (payload.orchestrator_url) {
    return payload.orchestrator_url.replace(/\/+$/, "");
  }

  const orchestrators = await discoverOrchestratorUrls();

  for (const orchestratorUrl of orchestrators) {
    const networks = await fetchNetworks(orchestratorUrl);
    const found = networks.find((network) => network.network_id === payload.network_id);
    if (found) {
      return (found.orchestrator_url || orchestratorUrl).replace(/\/+$/, "");
    }
  }

  return null;
}

export async function POST(req: NextRequest) {
  const payload = (await req.json()) as SubmitPayload;

  if (!payload.network_id || !payload.user_wallet || !payload.image) {
    return NextResponse.json(
      { error: "network_id, user_wallet and image are required" },
      { status: 400 },
    );
  }

  const targetOrchestrator = await resolveTargetOrchestrator(payload);
  if (!targetOrchestrator) {
    return NextResponse.json(
      {
        error: `No orchestrator found for network '${payload.network_id}'. Configure ORCHESTRATOR_URLS with all orchestrators.`,
      },
      { status: 404 },
    );
  }

  let upstream: Response;
  try {
    upstream = await fetch(`${targetOrchestrator}/jobs/submit`, {
      method: "POST",
      cache: "no-store",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify({
        network_id: payload.network_id,
        user_wallet: payload.user_wallet,
        image: payload.image,
        cpu_limit: payload.cpu_limit,
        ram_limit_mb: payload.ram_limit_mb,
        command: payload.command,
      }),
    });
  } catch {
    return NextResponse.json(
      { error: `Orchestrator unavailable at ${targetOrchestrator}` },
      { status: 503 },
    );
  }

  const raw = await upstream.text();
  return new NextResponse(raw, {
    status: upstream.status,
    headers: {
      "content-type": upstream.headers.get("content-type") ?? "application/json",
    },
  });
}
