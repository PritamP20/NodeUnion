export type Network = {
  network_id: string;
  name: string;
  description?: string;
  status: string;
  created_at_epoch_secs: number;
};

export type Node = {
  node_id: string;
  network_id: string;
  agent_url: string;
  provider_wallet?: string | null;
  status: string;
  is_idle: boolean;
  region?: string | null;
};

export type Job = {
  job_id: string;
  network_id: string;
  image: string;
  status: string;
  assigned_node_id?: string | null;
};

export type Entitlement = {
  entitlement_id: string;
  user_wallet: string;
  network_id: string;
  bought_units: number;
  used_units: number;
  escrow_tx_hash?: string | null;
};

export type Settlement = {
  settlement_id: string;
  job_id: string;
  network_id: string;
  units_metered: number;
  amount_tokens: number;
  tx_hash?: string | null;
  tx_status?: string | null;
  settlement_type?: string | null;
};

export type StellarProvider = {
  isPhantom?: boolean;
  isConnected?: boolean;
  publicKey?: { toString(): string };
  connect: () => Promise<unknown>;
  disconnect: () => Promise<void>;
};

declare global {
  interface Window {
    stellar?: StellarProvider;
  }
}

const API_BASE = "/api/orchestrator";

export async function fetchJson<T>(
  path: string,
  init?: RequestInit,
): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: {
      "content-type": "application/json",
      ...(init?.headers ?? {}),
    },
    cache: "no-store",
  });

  if (!response.ok) {
    const body = await response.text();
    throw new Error(body || `Request failed with status ${response.status}`);
  }

  return (await response.json()) as T;
}

export function shortenWallet(wallet: string): string {
  if (wallet.length <= 12) return wallet;
  return `${wallet.slice(0, 4)}...${wallet.slice(-4)}`;
}

export async function fetchOverview(): Promise<{
  networks: Network[];
  nodes: Node[];
  jobs: Job[];
}> {
  const [networks, nodes, jobs] = await Promise.all([
    fetchJson<Network[]>("/networks"),
    fetchJson<Node[]>("/nodes"),
    fetchJson<Job[]>("/jobs"),
  ]);

  return { networks, nodes, jobs };
}

export async function fetchWalletPortfolio(wallet: string): Promise<{
  entitlements: Entitlement[];
  settlements: Settlement[];
}> {
  const [entitlements, settlements] = await Promise.all([
    fetchJson<Entitlement[]>(`/users/${wallet}/entitlements`),
    fetchJson<Settlement[]>(`/users/${wallet}/settlements`),
  ]);

  return { entitlements, settlements };
}

export async function connectWallet(): Promise<string> {
  const provider = window.stellar;
  if (!provider) {
    throw new Error("No Stellar wallet found. Install a Stellar-compatible wallet.");
  }

  await provider.connect();
  const wallet = provider.publicKey?.toString();

  if (!wallet) {
    throw new Error("Wallet connected but no public key was returned.");
  }

  return wallet;
}

export async function disconnectWallet(): Promise<void> {
  const provider = window.stellar;
  if (provider?.disconnect) {
    await provider.disconnect();
  }
}
