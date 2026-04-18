# nodeunion-agent

NodeUnion provider agent daemon.

This service runs on provider machines, reports node health to the orchestrator, and executes assigned workloads as Docker containers.

## Install

```bash
cargo install nodeunion-agent
```

## Run

Set minimum environment variables:

```bash
export NODE_ID="provider-node-1"
export AGENT_BIND_ADDR="0.0.0.0:8090"
export ORCHESTRATOR_BASE_URL="http://127.0.0.1:8080"
```

Start the agent:

```bash
nodeunion-agent
```

## Local Monitor (TUI)

```bash
AGENT_URL="http://127.0.0.1:8090" cargo run --bin local_tui
```

## API Endpoints

- `GET /health`
- `GET /state`
- `POST /run`
- `POST /stop`
