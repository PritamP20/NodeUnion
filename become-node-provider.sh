#!/usr/bin/env bash
set -euo pipefail

print_usage() {
  cat <<'EOF'
NodeUnion Node Provider Bootstrap (one-command mode)

Usage:
  ./become-node-provider.sh --orchestrator-url <url> --network-id <id> [options]

Required:
  --orchestrator-url URL    Example: http://10.209.76.140:8080
  --network-id ID           Example: 11

Optional:
  --node-id ID
  --provider-wallet WALLET
  --bind-addr ADDR          Default: 0.0.0.0:8090
  --agent-public-url URL    Explicit public URL (skips auto tunnel)
  --public-url-provider P   cloudflare|none (default: cloudflare)
  --no-install              Do not auto-install dependencies
  -h, --help

Example:
  ./become-node-provider.sh --orchestrator-url http://10.209.76.140:8080 --network-id 11
EOF
}

log() {
  printf '[node-provider] %s\n' "$*"
}

fail() {
  printf '[node-provider] ERROR: %s\n' "$*" >&2
  exit 1
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

ORCHESTRATOR_URL="${ORCHESTRATOR_BASE_URL:-}"
NETWORK_ID="${NETWORK_ID:-}"
NODE_ID="${NODE_ID:-provider-$(hostname | tr '[:upper:]' '[:lower:]' | tr -cd 'a-z0-9-')-$(date +%s)}"
PROVIDER_WALLET="${PROVIDER_WALLET:-}"
BIND_ADDR="${AGENT_BIND_ADDR:-0.0.0.0:8090}"
AGENT_PUBLIC_URL_VALUE="${AGENT_PUBLIC_URL:-}"
AGENT_PUBLIC_URL_PROVIDER_VALUE="${AGENT_PUBLIC_URL_PROVIDER:-cloudflare}"
AUTO_INSTALL=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --orchestrator-url)
      ORCHESTRATOR_URL="${2:-}"
      shift 2
      ;;
    --network-id)
      NETWORK_ID="${2:-}"
      shift 2
      ;;
    --node-id)
      NODE_ID="${2:-}"
      shift 2
      ;;
    --provider-wallet)
      PROVIDER_WALLET="${2:-}"
      shift 2
      ;;
    --bind-addr)
      BIND_ADDR="${2:-}"
      shift 2
      ;;
    --agent-public-url)
      AGENT_PUBLIC_URL_VALUE="${2:-}"
      shift 2
      ;;
    --public-url-provider)
      AGENT_PUBLIC_URL_PROVIDER_VALUE="${2:-}"
      shift 2
      ;;
    --no-install)
      AUTO_INSTALL=0
      shift
      ;;
    -h|--help)
      print_usage
      exit 0
      ;;
    *)
      fail "Unknown argument: $1"
      ;;
  esac
done

[[ -n "$ORCHESTRATOR_URL" ]] || fail "--orchestrator-url is required"
[[ -n "$NETWORK_ID" ]] || fail "--network-id is required"

ORCHESTRATOR_URL="${ORCHESTRATOR_URL%/}"
if [[ "$ORCHESTRATOR_URL" != http://* && "$ORCHESTRATOR_URL" != https://* ]]; then
  ORCHESTRATOR_URL="http://$ORCHESTRATOR_URL"
fi

if ! have_cmd docker; then
  fail "Docker CLI not found. Install Docker Desktop first."
fi
if ! docker info >/dev/null 2>&1; then
  fail "Docker daemon is not running/reachable. Start Docker and retry."
fi

if [[ "$AUTO_INSTALL" -eq 1 ]] && ! have_cmd cargo; then
  if ! have_cmd curl; then
    fail "curl is required to install Rust automatically"
  fi
  log "Installing Rust (cargo) via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

have_cmd cargo || fail "cargo not found. Install Rust or rerun without --no-install."

if ! have_cmd nodeunion-agent; then
  if [[ "$AUTO_INSTALL" -eq 0 ]]; then
    fail "nodeunion-agent is missing. Remove --no-install to auto-install it."
  fi
  log "Installing nodeunion-agent..."
  cargo install nodeunion-agent --locked
fi

if [[ -z "$AGENT_PUBLIC_URL_VALUE" && "$AGENT_PUBLIC_URL_PROVIDER_VALUE" == "cloudflare" && ! $(have_cmd cloudflared; echo $?) -eq 0 ]]; then
  if [[ "$AUTO_INSTALL" -eq 0 ]]; then
    fail "cloudflared missing. Remove --no-install or pass --agent-public-url <public-url>."
  fi
  if have_cmd brew; then
    log "Installing cloudflared via brew..."
    brew install cloudflared
  elif have_cmd apt-get; then
    log "Installing cloudflared via apt-get..."
    sudo apt-get update
    sudo apt-get install -y cloudflared
  else
    fail "Cannot auto-install cloudflared on this OS. Pass --agent-public-url <public-url> instead."
  fi
fi

HEALTH_CODE="$(curl -sS -o /dev/null -w '%{http_code}' "$ORCHESTRATOR_URL/health" || true)"
if [[ "$HEALTH_CODE" != "200" ]]; then
  fail "Orchestrator is not healthy at $ORCHESTRATOR_URL (HTTP $HEALTH_CODE)"
fi

log "Starting NodeUnion agent"
log "node_id=$NODE_ID network_id=$NETWORK_ID orchestrator=$ORCHESTRATOR_URL bind_addr=$BIND_ADDR"

export NODE_ID="$NODE_ID"
export NETWORK_ID="$NETWORK_ID"
export PROVIDER_WALLET="$PROVIDER_WALLET"
export AGENT_BIND_ADDR="$BIND_ADDR"
export ORCHESTRATOR_BASE_URL="$ORCHESTRATOR_URL"
export AGENT_PUBLIC_URL_PROVIDER="$AGENT_PUBLIC_URL_PROVIDER_VALUE"
export AGENT_PUBLIC_URL="$AGENT_PUBLIC_URL_VALUE"

exec nodeunion-agent
