export type LandingFeature = {
  title: string;
  description: string;
  accent: string;
};

export type ComparisonRow = {
  vendor: string;
  costPerHour: string;
  setupTime: string;
  idleCost: string;
  payoutModel: string;
  best: boolean;
};

export type FlowStep = {
  title: string;
  description: string;
};

export type QuoteCard = {
  quote: string;
  author: string;
  role: string;
  monthlyEarnings: string;
};

export type RegionDot = {
  region: string;
  x: number;
  y: number;
  online: number;
};

export type NetworkNode = {
  nodeId: string;
  region: string;
  status: "Online" | "Offline" | "Draining";
  cpuUsagePct: number;
  ramUsageGb: number;
  lastHeartbeat: string;
  providerWallet: string;
  agentUrl?: string;
};

export type NetworkCard = {
  networkId: string;
  name: string;
  summary: string;
  orchestratorUrl?: string;
  onlineNodes: number;
  offlineNodes: number;
  activeJobs: number;
  totalComputeHours: number;
  sparkline: number[];
  dots: RegionDot[];
  nodes: NetworkNode[];
};

export type SeriesPoint = {
  label: string;
  value: number;
};

export type ProviderJob = {
  jobId: string;
  duration: string;
  cpuUsed: string;
  ramUsed: string;
  solEarned: string;
};

export type ProviderNode = {
  nodeId: string;
  networkId: string;
  region: string;
  status: string;
  uptime: string;
  jobsCompleted: number;
  agentUrl?: string;
  orchestratorUrl?: string;
};

export type UserJob = {
  jobId: string;
  image: string;
  network: string;
  status: "Running" | "Queued" | "Completed" | "Failed";
  cpuLimit: string;
  ramLimit: string;
  duration: string;
  credits: string;
};

export type DocSection = {
  id: string;
  group: string;
  title: string;
  summary: string;
  steps: string[];
  codeBlocks: Array<{
    label: string;
    language: string;
    code: string;
  }>;
  callout?: {
    tone: "info" | "tip" | "warning";
    title: string;
    body: string;
  };
};

export const landingTerminalSteps = [
  { input: "$ nodeunion jobs submit --network college-a", output: "Submitting job payload..." },
  { input: "wallet verified", output: "Wallet accepted. Checking entitlement credits..." },
  { input: "pull alpine:3.20", output: "Image pulled. Waiting for idle provider..." },
  { input: "assign node provider-node-1", output: "Node assigned. Booting container..." },
  { input: "deploy complete", output: "Container running. Settlement queued in XLM." },
];

export const landingFeatures: LandingFeature[] = [
  {
    title: "Decentralized Compute",
    description:
      "Idle GPUs and CPUs join a supply-side marketplace with real-time scheduling and node-aware routing.",
    accent: "from-sky-500/20 to-cyan-400/10",
  },
  {
    title: "Stellar-native Billing",
    description:
      "Usage, entitlements, and settlement records stay auditable with on-chain value movement.",
    accent: "from-emerald-500/20 to-lime-400/10",
  },
  {
    title: "Zero Idle Waste",
    description:
      "Provider machines only earn when they are actually doing useful work, not sitting dark.",
    accent: "from-indigo-500/20 to-blue-400/10",
  },
];

export const comparisonRows: ComparisonRow[] = [
  {
    vendor: "NodeUnion",
    costPerHour: "$0.00 when idle, paid in Stellar assets when busy",
    setupTime: "3 min",
    idleCost: "None",
    payoutModel: "Direct settlement to provider wallet",
    best: true,
  },
  {
    vendor: "AWS",
    costPerHour: "$2.40+",
    setupTime: "45 min",
    idleCost: "Always billed",
    payoutModel: "Invoice / card",
    best: false,
  },
  {
    vendor: "GCP",
    costPerHour: "$2.18+",
    setupTime: "38 min",
    idleCost: "Always billed",
    payoutModel: "Invoice / card",
    best: false,
  },
  {
    vendor: "Azure",
    costPerHour: "$2.31+",
    setupTime: "42 min",
    idleCost: "Always billed",
    payoutModel: "Invoice / card",
    best: false,
  },
];

export const flowSteps: FlowStep[] = [
  { title: "Machine Idle", description: "Provider node goes quiet and becomes schedulable." },
  { title: "Agent Detects", description: "Heartbeat and metrics confirm a healthy idle window." },
  { title: "Job Assigned", description: "Orchestrator chooses the best network-fit machine." },
  { title: "Compute Done", description: "Container runs to completion with metered usage." },
  { title: "Stellar Paid", description: "Provider receives settlement when usage is finalized." },
];

export const testimonialCards: QuoteCard[] = [
  {
    quote:
      "My workstation went from idle noise to steady XLM. I finally have a clean way to monetize spare GPU hours.",
    author: "Mara Chen",
    role: "Independent GPU provider",
    monthlyEarnings: "$1,840 / month",
  },
  {
    quote:
      "NodeUnion turned our lab cluster into a recurring revenue stream without locking us into a cloud contract.",
    author: "Darius Holt",
    role: "University compute admin",
    monthlyEarnings: "$3,120 / month",
  },
  {
    quote:
      "The payout is predictable, the machine utilization is transparent, and the billing trail is on-chain.",
    author: "Nina Shah",
    role: "Studio render provider",
    monthlyEarnings: "$2,410 / month",
  },
];

export const tickerStats = [
  { label: "Total nodes", value: "1,248" },
  { label: "Jobs run", value: "84,931" },
  { label: "XLM paid out", value: "18,420" },
];

export const networkCards: NetworkCard[] = [
  {
    networkId: "college-a",
    name: "College A Network",
    summary: "Research and lab workloads across three regions.",
    onlineNodes: 42,
    offlineNodes: 6,
    activeJobs: 18,
    totalComputeHours: 18420,
    sparkline: [22, 24, 20, 28, 30, 34, 31, 36, 38, 42],
    dots: [
      { region: "us-east-1", x: 68, y: 36, online: 18 },
      { region: "eu-west-1", x: 46, y: 28, online: 11 },
      { region: "ap-southeast-1", x: 82, y: 62, online: 13 },
    ],
    nodes: [
      {
        nodeId: "node-a-001",
        region: "us-east-1",
        status: "Online",
        cpuUsagePct: 34,
        ramUsageGb: 18,
        lastHeartbeat: "12s ago",
        providerWallet: "7qv9...mA3x",
      },
      {
        nodeId: "node-a-014",
        region: "eu-west-1",
        status: "Draining",
        cpuUsagePct: 62,
        ramUsageGb: 24,
        lastHeartbeat: "41s ago",
        providerWallet: "2Wf1...ZpL8",
      },
      {
        nodeId: "node-a-022",
        region: "ap-southeast-1",
        status: "Offline",
        cpuUsagePct: 0,
        ramUsageGb: 0,
        lastHeartbeat: "8m ago",
        providerWallet: "9dT3...Qk91",
      },
    ],
  },
  {
    networkId: "render-farm",
    name: "Render Farm",
    summary: "High-throughput visual compute with bursting demand.",
    onlineNodes: 68,
    offlineNodes: 4,
    activeJobs: 29,
    totalComputeHours: 29400,
    sparkline: [36, 38, 40, 41, 43, 46, 48, 50, 54, 57],
    dots: [
      { region: "us-west-2", x: 28, y: 38, online: 24 },
      { region: "us-central-1", x: 54, y: 47, online: 17 },
      { region: "eu-central-1", x: 62, y: 30, online: 27 },
    ],
    nodes: [
      {
        nodeId: "node-r-002",
        region: "us-west-2",
        status: "Online",
        cpuUsagePct: 51,
        ramUsageGb: 28,
        lastHeartbeat: "7s ago",
        providerWallet: "4cP9...F9s2",
      },
      {
        nodeId: "node-r-019",
        region: "eu-central-1",
        status: "Online",
        cpuUsagePct: 45,
        ramUsageGb: 31,
        lastHeartbeat: "19s ago",
        providerWallet: "8aJ4...tC71",
      },
      {
        nodeId: "node-r-031",
        region: "us-central-1",
        status: "Draining",
        cpuUsagePct: 71,
        ramUsageGb: 26,
        lastHeartbeat: "24s ago",
        providerWallet: "1kS7...xG35",
      },
    ],
  },
  {
    networkId: "gpu-club",
    name: "GPU Club",
    summary: "Community-operated pool for burst AI workloads.",
    onlineNodes: 34,
    offlineNodes: 8,
    activeJobs: 13,
    totalComputeHours: 9300,
    sparkline: [12, 14, 15, 16, 17, 18, 19, 21, 23, 25],
    dots: [
      { region: "ca-central-1", x: 22, y: 30, online: 9 },
      { region: "us-east-1", x: 38, y: 42, online: 14 },
      { region: "sa-east-1", x: 54, y: 72, online: 11 },
    ],
    nodes: [
      {
        nodeId: "node-g-004",
        region: "us-east-1",
        status: "Online",
        cpuUsagePct: 27,
        ramUsageGb: 12,
        lastHeartbeat: "11s ago",
        providerWallet: "6pK2...Lr60",
      },
      {
        nodeId: "node-g-010",
        region: "ca-central-1",
        status: "Online",
        cpuUsagePct: 22,
        ramUsageGb: 16,
        lastHeartbeat: "22s ago",
        providerWallet: "3uQ6...Vn44",
      },
      {
        nodeId: "node-g-016",
        region: "sa-east-1",
        status: "Offline",
        cpuUsagePct: 0,
        ramUsageGb: 0,
        lastHeartbeat: "14m ago",
        providerWallet: "7yR8...dA92",
      },
    ],
  },
];

export const providerEarningsSeries: SeriesPoint[] = Array.from({ length: 30 }, (_, index) => ({
  label: `D${index + 1}`,
  value: 8 + ((index * 7) % 17) + (index % 3) * 4,
}));

export const providerJobs: ProviderJob[] = [
  { jobId: "job-91af", duration: "58m", cpuUsed: "3.4", ramUsed: "12 GB", solEarned: "1.42" },
  { jobId: "job-7d31", duration: "41m", cpuUsed: "2.1", ramUsed: "8 GB", solEarned: "0.97" },
  { jobId: "job-2ca8", duration: "2h 12m", cpuUsed: "6.9", ramUsed: "24 GB", solEarned: "3.16" },
  { jobId: "job-0f11", duration: "33m", cpuUsed: "1.8", ramUsed: "6 GB", solEarned: "0.81" },
];

export const providerNodes: ProviderNode[] = [
  { nodeId: "provider-node-1", networkId: "college-a", region: "us-east-1", status: "Online", uptime: "99.8%", jobsCompleted: 48 },
  { nodeId: "provider-node-2", networkId: "render-farm", region: "eu-central-1", status: "Online", uptime: "98.9%", jobsCompleted: 36 },
  { nodeId: "provider-node-3", networkId: "gpu-club", region: "ca-central-1", status: "Draining", uptime: "96.7%", jobsCompleted: 21 },
];

export const userSpendingSeries: SeriesPoint[] = Array.from({ length: 30 }, (_, index) => ({
  label: `D${index + 1}`,
  value: 3 + ((index * 5) % 8) + (index % 4),
}));

export const userJobs: UserJob[] = [
  { jobId: "job-4f20", image: "python:3.11-alpine", network: "college-a", status: "Completed", cpuLimit: "0.5", ramLimit: "512 MB", duration: "28m", credits: "7.2" },
  { jobId: "job-77ab", image: "alpine:3.20", network: "render-farm", status: "Running", cpuLimit: "1.0", ramLimit: "1 GB", duration: "14m", credits: "2.8" },
  { jobId: "job-c001", image: "node:20-alpine", network: "gpu-club", status: "Queued", cpuLimit: "0.25", ramLimit: "256 MB", duration: "--", credits: "0.0" },
  { jobId: "job-1a9f", image: "ubuntu:24.04", network: "college-a", status: "Failed", cpuLimit: "2.0", ramLimit: "4 GB", duration: "11m", credits: "1.1" },
];

export const providerSummary = {
  totalEarned: "184.6 XLM",
  earnedMonth: "37.1 XLM",
  pendingSettlement: "8.9 XLM",
  uptime: "99.3%",
  jobsCompleted: 148,
  rejectionRate: "1.2%",
};

export const userSummary = {
  creditsRemaining: "12,480",
  spentMonth: "3,860",
  totalSpend: "15,220",
};

export const docSections: DocSection[] = [
  {
    id: "getting-started",
    group: "Getting Started",
    title: "Prerequisites and local install",
    summary: "Install the CLI stack and verify the machine is ready for dev/test deployment.",
    steps: [
      "Install Rust stable, Node.js 20+, Docker, Stellar CLI, and PostgreSQL.",
      "Verify each tool with version commands before touching the stack.",
      "Use cargo install nodeunion-agent and cargo install nodeunion-orchestrator for local binaries.",
    ],
    codeBlocks: [
      {
        label: "Install binaries",
        language: "bash",
        code: "cargo install nodeunion-agent\ncargo install nodeunion-orchestrator",
      },
      {
        label: "Verify toolchain",
        language: "bash",
        code: "rustc --version\ncargo --version\nnode --version\ndocker --version",
      },
    ],
    callout: {
      tone: "info",
      title: "Current deployability",
      body: "The current stack is intended for dev/test today: orchestrator on 8080, agent on 8090, dashboard on 3000, and Stellar billing on testnet.",
    },
  },
  {
    id: "stellar-program",
    group: "Orchestrator Setup",
    title: "Deploy the Stellar billing contract",
    summary: "Build and deploy the Soroban contract on testnet, then wire the contract ID into orchestrator config.",
    steps: [
      "Ensure the Stellar CLI is installed and the testnet key is funded.",
      "Build the Soroban workspace.",
      "Deploy the contract to testnet.",
      "Copy the deployed contract ID into STELLAR_CONTRACT_ID.",
    ],
    codeBlocks: [
      {
        label: "Devnet deploy",
        language: "bash",
        code: "cd stellar-container\ncargo build --target wasm32-unknown-unknown --release\nstellar contract deploy \\\n  --wasm target/wasm32-unknown-unknown/release/nodeunion_billing.wasm \\\n  --network testnet \\\n  --source-account nodeunion-test",
      },
    ],
    callout: {
      tone: "tip",
      title: "Keep the source account funded",
      body: "Billing settlement will fail if the signer identity is unfunded or the contract ID does not match the deployed Soroban artifact.",
    },
  },
  {
    id: "orchestrator-setup",
    group: "Orchestrator Setup",
    title: "Start the orchestrator and dashboard",
    summary: "Run the control plane with PostgreSQL and expose the dashboard through the proxy route.",
    steps: [
      "Set DATABASE_URL, Stellar contract settings, source account, and single-network metadata.",
      "Start the orchestrator and confirm /health responds on port 8080.",
      "Run the dashboard with ORCHESTRATOR_URL pointing at the orchestrator host.",
    ],
    codeBlocks: [
      {
        label: "Orchestrator env",
        language: "bash",
        code: "export DATABASE_URL=\"postgresql://USER:PASSWORD@HOST:5432/DBNAME?sslmode=require\"\nexport STELLAR_NETWORK=\"mainnet\"\nexport STELLAR_SOURCE_ACCOUNT=\"nodeunion-test\"\nexport STELLAR_CONTRACT_ID=\"CC5DFOTE24IDJPFL5IV4647TAAZYCOCJEO4UR76SZPFIBTCTBKPXKV2K\"\nexport STELLAR_RATE_PER_UNIT=\"100\"\nexport ORCHESTRATOR_BIND_ADDR=\"0.0.0.0:8080\"\nexport ORCHESTRATOR_NETWORK_ID=\"college-a\"\nexport ORCHESTRATOR_NETWORK_NAME=\"College A Network\"",
      },
      {
        label: "Run services",
        language: "bash",
        code: "cargo run -p orchestrator\ncd dashboard\nexport ORCHESTRATOR_URL=\"http://127.0.0.1:8080\"\nnpm run dev",
      },
    ],
    callout: {
      tone: "warning",
      title: "Single-network mode",
      body: "When ORCHESTRATOR_NETWORK_ID is set, the control plane only accepts and serves that network, which keeps dashboard and agent views consistent.",
    },
  },
  {
    id: "agent-setup",
    group: "Agent Setup",
    title: "Register providers and launch the agent",
    summary: "Provider machines register with wallet, network, and reachability metadata, then start heartbeating.",
    steps: [
      "Create a provider .env or use the launch TUI.",
      "Set NODE_ID, NETWORK_ID, AGENT_BIND_ADDR, and ORCHESTRATOR_BASE_URL.",
      "Start the agent on a Docker-enabled machine and confirm it sends heartbeats.",
    ],
    codeBlocks: [
      {
        label: "Agent env",
        language: "bash",
        code: "NODE_ID=provider-node-1\nNETWORK_ID=college-a\nAGENT_BIND_ADDR=0.0.0.0:8090\nORCHESTRATOR_BASE_URL=http://127.0.0.1:8080\nHEARTBEAT_INTERVAL_SECS=60\nMETRICS_POLL_INTERVAL_SECS=30",
      },
      {
        label: "Register provider",
        language: "bash",
        code: `curl -sS -X POST http://127.0.0.1:8080/nodes/register \
  -H 'content-type: application/json' \
  -d '{
    "node_id": "provider-node-1",
    "network_id": "college-a",
    "agent_url": "http://127.0.0.1:8090",
    "provider_wallet": "<PROVIDER_STELLAR_WALLET>",
    "region": "us-east-1"
  }'`,
      },
    ],
    callout: {
      tone: "tip",
      title: "Agent reachability matters",
      body: "The orchestrator must be able to reach each agent at its registered agent_url to dispatch workloads successfully.",
    },
  },
  {
    id: "job-submission",
    group: "Job Submission",
    title: "Submit workloads and monitor state",
    summary: "Users submit image + command payloads with wallet identity and network selection.",
    steps: [
      "Choose a target network that has idle providers.",
      "Attach wallet identity and image/command payload.",
      "Submit the job and monitor /jobs and the dashboard queue.",
    ],
    codeBlocks: [
      {
        label: "Submit job",
        language: "bash",
        code: `curl -sS -X POST http://127.0.0.1:8080/jobs/submit \
  -H 'content-type: application/json' \
  -d '{
    "network_id": "college-a",
    "user_wallet": "<USER_STELLAR_WALLET>",
    "image": "alpine:3.20",
    "command": ["echo", "hello-nodeunion"],
    "cpu_limit": 0.25,
    "ram_limit_mb": 128
  }'`,
      },
    ],
  },
  {
    id: "stellar-billing",
    group: "Stellar Billing",
    title: "Credits, entitlements, and payout flow",
    summary: "User entitlements are tracked in PostgreSQL and usage settles on-chain through the Stellar billing contract.",
    steps: [
      "Seed or top up user_entitlements for a wallet and network.",
      "Track settlement records and signature status after job completion.",
      "Keep payout and payer credentials in a secret manager for production.",
    ],
    codeBlocks: [
      {
        label: "Top up entitlement",
        language: "sql",
        code: "INSERT INTO user_entitlements (entitlement_id, user_wallet, network_id, bought_units, used_units, created_at_epoch_secs)\nVALUES ('entl-user1-college-a', '<USER_STELLAR_WALLET>', 'college-a', 100000, 0, EXTRACT(EPOCH FROM NOW())::BIGINT)\nON CONFLICT (user_wallet, network_id)\nDO UPDATE SET bought_units = user_entitlements.bought_units + EXCLUDED.bought_units;",
      },
    ],
    callout: {
      tone: "info",
      title: "Billing guardrail",
      body: "The orchestrator requires entitlement credits before accepting jobs unless the local testing bypass is enabled.",
    },
  },
  {
    id: "api-reference",
    group: "API Reference",
    title: "REST endpoints",
    summary: "The dashboard and operators rely on a small set of JSON endpoints for networks, nodes, jobs, and wallets.",
    steps: [
      "GET /health, /networks, /nodes, and /jobs for live state.",
      "POST /networks/create, /nodes/register, and /jobs/submit for control-plane actions.",
      "GET /users/:wallet/entitlements and /users/:wallet/settlements for wallet views.",
    ],
    codeBlocks: [
      {
        label: "Health check",
        language: "bash",
        code: "curl -i http://127.0.0.1:8080/health",
      },
      {
        label: "Networks response",
        language: "json",
        code: "[\n  {\n    \"network_id\": \"college-a\",\n    \"name\": \"College A Network\",\n    \"status\": \"Active\"\n  }\n]",
      },
    ],
  },
  {
    id: "troubleshooting",
    group: "Troubleshooting",
    title: "Common failure modes",
    summary: "When the stack misbehaves, check the backend, Docker reachability, and network metadata first.",
    steps: [
      "402 on /jobs/submit usually means the wallet lacks entitlement credits.",
      "Node offline often means the agent cannot reach the orchestrator URL.",
      "Settlement failures usually point to a mismatched program ID or a drained payer wallet.",
    ],
    codeBlocks: [
      {
        label: "Monitor state",
        language: "bash",
        code: "curl -sS http://127.0.0.1:8080/networks | jq\ncurl -sS http://127.0.0.1:8080/nodes | jq\ncurl -sS http://127.0.0.1:8080/jobs | jq",
      },
    ],
  },
];
