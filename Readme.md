# NodeUnion Deployment Guide (Devnet)

This document explains how to deploy the current NodeUnion stack and how each actor uses it:

- Platform operator: deploys Solana program, orchestrator, and dashboard
- Compute provider: runs agent and registers provider wallet
- User: submits jobs with wallet and compute credits

## Current Deployability

Yes, you can deploy this now for dev/test.

- Orchestrator runs on port 8080 and persists state in PostgreSQL.
- Agent runs on port 8090 and sends heartbeats to orchestrator.
- Dashboard proxies to orchestrator and can be run on port 3000.
- Solana billing program can be deployed to devnet via Anchor.

## Local Install Via crates.io

For provider machines, the agent can be distributed as a Cargo-installed binary.
For self-hosted deployments, the orchestrator can also be installed the same way.

```bash
cargo install nodeunion-agent
cargo install nodeunion-orchestrator
```

Use the orchestrator as the central public control plane and run the agent on each compute provider machine.
The shared DB layer is bundled inside the orchestrator crate, so providers do not need a separate database package.

## Local TUI Monitor

To watch orchestrator state locally in the terminal:

```bash
cd crates/orchestrator
ORCHESTRATOR_URL="http://127.0.0.1:8080" cargo run --bin local_tui
```

The monitor auto-refreshes every 2 seconds and shows health, networks, nodes, jobs, and recent errors.

For an agent-local monitor (node status, metrics, running chunks):

```bash
cd crates/agent
AGENT_URL="http://127.0.0.1:8090" cargo run --bin local_tui
```

This monitor auto-refreshes every 2 seconds and reads `/health` and `/state` from the agent.

## Interactive Launch And Deploy TUIs

You can now use simple terminal forms to start orchestrator, start agent, and submit user jobs.

### Orchestrator launch form

```bash
cd crates/orchestrator
cargo run --bin nodeunion-orchestrator-launch-tui
```

This asks for required orchestrator env fields and then starts `nodeunion-orchestrator`.
It also prints a detected reachable orchestrator URL (IP + port) that you can share with agents/users.

### Agent launch form

```bash
cd crates/agent
cargo run --bin nodeunion-agent-launch-tui
```

This asks for required agent env fields and then starts `nodeunion-agent`.

### User deploy form

```bash
cd crates/orchestrator
cargo run --bin nodeunion-orchestrator-user-tui
```

This flow will:

1. Fetch available networks from orchestrator
2. Show node count per network
3. Let user select a network
4. Ask for Dockerfile path
5. Build the image locally with Docker
6. Optionally push the image to a registry (recommended for remote providers)
7. Submit a job to orchestrator using that image tag

## Prerequisites

Install:

- Rust stable + Cargo
- Node.js 20+
- Docker (for provider machines)
- Solana CLI
- Anchor CLI
- PostgreSQL (local or Neon)

Verify tools:

```bash
rustc --version
cargo --version
node --version
solana --version
anchor --version
docker --version
```

## 1) Deploy the Solana Program to Devnet

From the solana workspace:

```bash
cd solana
solana config set --url https://api.devnet.solana.com
solana airdrop 2
anchor build
anchor deploy --provider.cluster devnet
```

Get deployed program id:

```bash
solana address -k target/deploy/nodeunion_billing-keypair.json
```

Keep this value for SOLANA_PROGRAM_ID.

## 2) Start the Orchestrator

Create env file for orchestrator shell session:

```bash
export DATABASE_URL="postgresql://USER:PASSWORD@HOST:5432/DBNAME?sslmode=require"
export SOLANA_RPC_URL="https://api.devnet.solana.com"
export SOLANA_PAYER_KEYPAIR="$HOME/.config/solana/id.json"
export SOLANA_PROGRAM_ID="<PASTE_DEPLOYED_PROGRAM_ID>"
export ORCHESTRATOR_BIND_ADDR="0.0.0.0:8080"
export ORCHESTRATOR_NETWORK_ID="college-a"
export ORCHESTRATOR_NETWORK_NAME="College A Network"
export ORCHESTRATOR_NETWORK_DESCRIPTION="Primary network for college-a"
export ORCHESTRATOR_PUBLIC_URL="https://api.nodeunion.ai"
```

Run orchestrator:

```bash
cargo run -p orchestrator
```

`ORCHESTRATOR_NETWORK_ID` enables single-network mode. When set, this orchestrator only accepts and serves that one network.
On startup, orchestrator upserts this managed network in DB, including `ORCHESTRATOR_PUBLIC_URL`, so user tooling can discover the network entry and its control-plane URL.

Health check:

```bash
curl -i http://127.0.0.1:8080/health
```

## 3) Start the Dashboard

In a new terminal:

```bash
cd dashboard
export ORCHESTRATOR_URL="http://127.0.0.1:8080"
npm install
npm run dev
```

Open http://127.0.0.1:3000.

## 4) Start a Compute Provider Agent

On each provider machine:

```bash
cd crates/agent
cp .env.example .env
```

Set minimum values in .env:

```env
NODE_ID=provider-node-1
AGENT_BIND_ADDR=0.0.0.0:8090
ORCHESTRATOR_BASE_URL=http://<ORCHESTRATOR_HOST>:8080
HEARTBEAT_INTERVAL_SECS=60
METRICS_POLL_INTERVAL_SECS=30
IDLE_CPU_THRESHOLD_PCT=15.0
PREEMPT_CPU_THRESHOLD_PCT=60.0
IDLE_WINDOW_SAMPLES=10
REQUEST_TIMEOUT_SECS=30
```

Run provider agent:

```bash
cargo run -p agent
```

## 5) Platform Bootstrapping Sequence

Run these in order once orchestrator is up.

### Create network

```bash
curl -sS -X POST http://127.0.0.1:8080/networks/create \
       -H "content-type: application/json" \
       -d '{
              "network_id": "college-a",
              "name": "College A Network",
              "description": "Devnet pilot"
       }'
```

### Register provider node with payout wallet

```bash
curl -sS -X POST http://127.0.0.1:8080/nodes/register \
       -H "content-type: application/json" \
       -d '{
              "node_id": "provider-node-1",
              "network_id": "college-a",
              "agent_url": "http://127.0.0.1:8090",
              "provider_wallet": "<PROVIDER_SOLANA_WALLET>",
              "region": "us-east-1"
       }'
```

### Add user entitlement (required before submit)

The current API enforces credits via the user_entitlements table before accepting jobs.
Until top-up endpoints are added, seed credits directly in PostgreSQL:

```sql
INSERT INTO user_entitlements (
       entitlement_id,
       user_wallet,
       network_id,
       bought_units,
       used_units,
       created_at_epoch_secs
) VALUES (
       'entl-user1-college-a',
       '<USER_SOLANA_WALLET>',
       'college-a',
       100000,
       0,
       EXTRACT(EPOCH FROM NOW())::BIGINT
)
ON CONFLICT (user_wallet, network_id)
DO UPDATE SET bought_units = user_entitlements.bought_units + EXCLUDED.bought_units;
```

### Submit user job

```bash
curl -sS -X POST http://127.0.0.1:8080/jobs/submit \
       -H "content-type: application/json" \
       -d '{
              "network_id": "college-a",
              "user_wallet": "<USER_SOLANA_WALLET>",
              "image": "alpine:3.20",
              "command": ["echo", "hello-nodeunion"],
              "cpu_limit": 0.25,
              "ram_limit_mb": 128
       }'
```

### Monitor state

```bash
curl -sS http://127.0.0.1:8080/networks | jq
curl -sS http://127.0.0.1:8080/nodes | jq
curl -sS http://127.0.0.1:8080/jobs | jq
```

## Compute Provider Workflow

Each provider does this:

1. Runs the agent on a machine with Docker.
2. Shares node_id, agent_url, and provider_wallet during registration.
3. Agent sends heartbeat to orchestrator and receives /run jobs.
4. Provider earns payouts when usage settlement calls are finalized on-chain.

Provider checklist:

- Docker daemon is reachable.
- Agent can reach orchestrator URL.
- Provider wallet is correct and owned by provider.

## User Workflow

Each user does this:

1. Chooses a network.
2. Uses a Solana wallet address as identity in job submit requests.
3. Must have entitlement credits in that network.
4. Submits image + resource limits.
5. Tracks status from /jobs or dashboard.

User checklist:

- Wallet address is valid base58 Solana pubkey.
- Credits exist in user_entitlements for target network.
- Job image is pullable by provider node Docker.

## Production Notes

- Store SOLANA_PAYER_KEYPAIR in a secret manager, not plain files.

## Public Internet Deployment (Global Access)

If you want users to submit jobs from anywhere in the world, deploy the orchestrator behind a public HTTPS domain.

### Public control-plane pattern

1. Run `nodeunion-orchestrator` on a cloud VM/container with:

```bash
export ORCHESTRATOR_BIND_ADDR="0.0.0.0:8080"
```

2. Put Nginx/Caddy/ALB in front and expose only `443`.
3. Map your domain (for example: `api.nodeunion.ai`) to the proxy.
4. Proxy `https://api.nodeunion.ai` to orchestrator `http://127.0.0.1:8080`.

### Provider connectivity

For providers in different regions/networks, each agent must be reachable by the orchestrator at its registered `agent_url`.

Recommended options:

1. Provider exposes agent over HTTPS with firewall allowlist limited to orchestrator IP.
2. Provider uses a secure tunnel (Cloudflare Tunnel / Tailscale Funnel / reverse proxy tunnel) and registers that URL as `agent_url`.

### User submission endpoint

Once deployed, users submit jobs to your public endpoint instead of localhost:

```bash
curl -sS -X POST https://api.nodeunion.ai/jobs/submit \
       -H "content-type: application/json" \
       -d '{
              "network_id": "college-a",
              "user_wallet": "<USER_SOLANA_WALLET>",
              "image": "python:3.11-alpine",
              "command": ["python", "-c", "print(\"hello from anywhere\")"],
              "cpu_limit": 0.25,
              "ram_limit_mb": 128
       }'
```

The agent will pull missing images automatically before starting containers.
- Move from direct SQL entitlement seeding to dedicated top-up API.
- Enable TLS and private networking between orchestrator and agents.
- Add retries and confirmation checks for all Solana tx signatures.

## Troubleshooting

402 Payment Required on /jobs/submit:

- User has no entitlement row or insufficient units.

Node remains offline:

- Verify agent heartbeat can reach orchestrator URL.

Solana transaction failures:

- Check SOLANA_PROGRAM_ID matches deployed program.
- Check payer wallet has devnet SOL balance.
- Check orchestrator logs for RPC error payload.

## What the User Sees

1. Sign in with institutional email
2. Upload Docker image or paste image URL
3. Set CPU and RAM requirements
4. Click Deploy
5. Get a public URL within 60 seconds
6. App stays online — migrates silently between idle machines
7. Pay only for actual compute time, billed in IDLE tokens on Solana
8. When no machines are available, app queues and auto-resumes

---

*Built with Rust. Runs on machines that would otherwise be doing nothing.*

pritamp20@Pritams-MacBook-Air solana %  solana balance --url https://api.devnet.solana.com && anchor deploy --provider.cluster devnet 2>&1 | tail -120
5.78439508 SOL
Deploying cluster: https://api.devnet.solana.com
Upgrade authority: /Users/pritamp20/.config/solana/id.json
Deploying program "nodeunion_billing"...
Program path: /Users/pritamp20/Documents/NodeUnion/solana/target/deploy/nodeunion_billing.so...
Program Id: zEmSJV8TWmSwHX2f6RdyFvZgvCwQaJ9ZrLfdQtidexo

Signature: 4anJjiqCuKQvnR1UWvZrtWxguhA8Qyqhxvm727rnCZJ13gg1sUsZgkbuKEW6K3jSXsbgBkFkuttC97D9UiUqGoCW

Deploy success
pritamp20@Pritams-MacBook-Air solana % 