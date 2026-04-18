# nodeunion-orchestrator

NodeUnion orchestration backend.

This service manages networks, nodes, jobs, and settlement-related state for the NodeUnion platform.

## Install

```bash
cargo install nodeunion-orchestrator
```

## Run

Set minimum environment variables:

```bash
export DATABASE_URL="postgresql://USER:PASSWORD@HOST:5432/DBNAME?sslmode=require"
export SOLANA_RPC_URL="https://api.devnet.solana.com"
export SOLANA_PAYER_KEYPAIR="$HOME/.config/solana/id.json"
export SOLANA_PROGRAM_ID="<PROGRAM_ID>"
```

Start the orchestrator:

```bash
nodeunion-orchestrator
```

## Local Monitor (TUI)

```bash
ORCHESTRATOR_URL="http://127.0.0.1:8080" cargo run --bin local_tui
```

## Core API Endpoints

- `GET /health`
- `GET /networks`
- `GET /nodes`
- `GET /jobs`
- `POST /networks/create`
- `POST /nodes/register`
- `POST /jobs/submit`
