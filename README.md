# NodeUnion — Decentralized Compute Billing (Soroban / Stellar)

A minimal, production-oriented orchestrator and provider agent system that bills compute usage on Soroban (Stellar). This repository contains the orchestrator, agent, DB schema, and dashboard used to schedule containerized jobs, meter compute, and settle payments on-chain via a Soroban billing contract.

Key components
- `crates/orchestrator` — HTTP API, job scheduler, Stellar client integration, billing logic.
- `crates/agent` — compute provider that executes container jobs (Docker), reports telemetry, and receives payments.
- `crates/db` — database access layer and entitlements repository.
- `dashboard/` — Next.js UI for submitting deployments, monitoring jobs and balances.
- `stellar-container/contracts/` — Soroban contract sources and compiled WASM (testnet deploys).

Highlights
- Soroban (Stellar) billing contract integration (testnet deployed; mainnet-ready workflow).
- TUI launchers for quick local development: orchestrator and agent launch TUIs and local monitoring TUIs.
- Database-first design with entitlements and usage tracking for metered billing.

Quick start (local development)

Requirements
- Rust + Cargo (with `wasm32-unknown-unknown` target for contract builds)
- Docker (for agent job execution)
- PostgreSQL (local or remote Neon)
- Stellar CLI (`stellar` / `soroban` CLI) configured for the network you use

1) Build and install TUIs (from repository root):

```bash
cargo build --release -p nodeunion-orchestrator -p nodeunion-agent
mkdir -p ~/.local/nodeunion/bin
cp target/release/nodeunion-* ~/.local/nodeunion/bin/
export PATH="$HOME/.local/nodeunion/bin:$PATH"
```

2) Start local Postgres (optional):

```bash
docker run -d --name nodeunion-postgres \
  -e POSTGRES_USER=nodeunion \
  -e POSTGRES_PASSWORD=nodeunion \
  -e POSTGRES_DB=nodeunion \
  -p 5432:5432 postgres:15
```

3) Launch orchestrator via TUI and fill values from `TUI_CONFIGURATION.md`:

```bash
nodeunion-orchestrator-launch-tui
```

4) Launch an agent (in a separate terminal):

```bash
nodeunion-agent-launch-tui
```

5) Use the dashboard to submit deployments and monitor jobs (run `dashboard` with `npm install && npm run dev`).

Configuration
- See `TUI_CONFIGURATION.md` for all environment variables and recommended values for testnet.
- Main orchestration environment variables (example):

```bash
export DATABASE_URL="postgres://nodeunion:nodeunion@localhost:5432/nodeunion"
export STELLAR_NETWORK="TESTNET_FUTURE"
export STELLAR_SOURCE_ACCOUNT="GDWUXVSRV..."
export STELLAR_CONTRACT_ID="CC5DFOTE24IDJPFL5IV4647TAAZYCOCJEO4UR76SZPFIBTCTBKPXKV2K"
```

Deploying Soroban contract to Mainnet (summary & precautions)
- Yes — the project supports deploying the Soroban billing contract to mainnet. Before doing so:
  - Audit the contract code and review storage/rent implications.
  - Fund your deployer/source account with a sufficient XLM buffer (recommended 100 XLM as a safe starting buffer; see `stellar` CLI cost estimate before submitting).
  - Ensure you control the private keys securely and use hardware or secure key management for production.
  - Update `STELLAR_CONTRACT_ID` and `STELLAR_SOURCE_ACCOUNT` in orchestrator configuration and restart services.

Project development notes
- Billing checks are enforced by the `user_entitlements` table. For local testing you can top up entitlements directly via SQL or set `DISABLE_BILLING_CHECK=1` (only for local testing).
- `crates/orchestrator/src/stellar_client.rs` uses the Stellar CLI for contract invokes. Production deployments should consider integrating a direct RPC or SDK flow and secure key handling.

Contributing
- Fork the repo, open a feature branch, and send a PR. Run unit tests and TUI checks before submitting.

Useful files
- `TUI_CONFIGURATION.md` — local TUI config and values
- `LOCAL_DEPLOYMENT.md` — notes about how binaries were installed locally
- `stellar-container/contracts/` — Soroban contract source & build scripts

Contact
- Open issues or PRs on this repository for questions and patches.

License
- This repository does not include a license file. Add a `LICENSE` if you intend to open-source.
