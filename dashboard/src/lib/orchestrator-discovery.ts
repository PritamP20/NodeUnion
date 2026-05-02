type DiscoveryNetworkRecord = {
  orchestrator_url?: string | null;
};

function normalizeBaseUrl(raw: string) {
  const value = raw.trim();
  if (!value) return null;

  try {
    const parsed = new URL(value);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
      return null;
    }
    return `${parsed.origin}${parsed.pathname}`.replace(/\/+$/, "");
  } catch {
    return null;
  }
}

export function getSeedOrchestratorUrls() {
  const many = process.env.ORCHESTRATOR_URLS
    ?.split(",")
    .map((value) => normalizeBaseUrl(value))
    .filter((value): value is string => Boolean(value));

  if (many && many.length > 0) {
    return Array.from(new Set(many));
  }

  const single = normalizeBaseUrl(process.env.ORCHESTRATOR_URL ?? "http://127.0.0.1:8080");
  return single ? [single] : ["http://127.0.0.1:8080"];
}

async function fetchNetworks(baseUrl: string, timeoutMs: number) {
  const response = await fetch(`${baseUrl}/networks`, {
    cache: "no-store",
    signal: AbortSignal.timeout(timeoutMs),
  });

  if (!response.ok) {
    throw new Error(`networks fetch failed with ${response.status}`);
  }

  return (await response.json()) as DiscoveryNetworkRecord[];
}

export async function discoverOrchestratorUrls() {
  const maxSources = Number.parseInt(process.env.MAIN_ORCHESTRATOR_DISCOVERY_LIMIT ?? "48", 10);
  const timeoutMs = Number.parseInt(process.env.MAIN_ORCHESTRATOR_DISCOVERY_TIMEOUT_MS ?? "2500", 10);

  const queue = [...getSeedOrchestratorUrls()];
  const seen = new Set<string>();

  while (queue.length > 0 && seen.size < maxSources) {
    const candidate = queue.shift();
    if (!candidate || seen.has(candidate)) {
      continue;
    }

    seen.add(candidate);

    try {
      const networks = await fetchNetworks(candidate, timeoutMs);
      for (const network of networks) {
        const discovered = normalizeBaseUrl(network.orchestrator_url ?? "");
        if (discovered && !seen.has(discovered)) {
          queue.push(discovered);
        }
      }
    } catch {
      // Ignore failures here. Health and error details are captured in snapshot source statuses.
    }
  }

  return Array.from(seen);
}
