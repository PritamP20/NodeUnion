# nodeunion-agent

NodeUnion provider agent daemon.

This service runs on provider machines, reports node health to the orchestrator, and executes assigned workloads as Docker containers.

## Install

```bash
cargo install nodeunion-agent
```

## One-Line Provider Start

After install, a provider can start in one command (auto public URL via Cloudflare tunnel):

```bash
nodeunion-agent-quickstart --orchestrator-url http://<ORCHESTRATOR_HOST>:8080 --network-id <NETWORK_ID>
```

If you already have a public URL, pass it directly:

```bash
nodeunion-agent-quickstart --orchestrator-url http://<ORCHESTRATOR_HOST>:8080 --network-id <NETWORK_ID> --agent-public-url https://<PUBLIC_AGENT_URL>
```

## Run

Set minimum environment variables:

```bash
export NODE_ID="provider-node-1"
export AGENT_BIND_ADDR="0.0.0.0:8090"
export ORCHESTRATOR_BASE_URL="http://127.0.0.1:8080"
```

Optional public URL settings:

```bash
# Explicit public URL (recommended when you already have a tunnel URL)
export AGENT_PUBLIC_URL="https://your-public-agent-url.example.com"

# Or let the agent auto-create a Cloudflare quick tunnel and use that URL
export AGENT_PUBLIC_URL_PROVIDER="cloudflare"
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
