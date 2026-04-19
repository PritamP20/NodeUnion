#!/bin/bash
set -e

cat <<'EOF'
NodeUnion Node Provider Bootstrap

This script will launch the agent configuration wizard.
Answer the prompts to set up your node provider.
EOF

echo ""

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: Docker is required for provider nodes, but the Docker CLI is not installed."
  echo "Install Docker Desktop and make sure the daemon is running before starting a node."
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "ERROR: Docker is installed, but the Docker daemon is not running or not reachable."
  echo "Start Docker Desktop / the Docker service, then run this script again."
  exit 1
fi

if command -v nodeunion-agent-launch-tui >/dev/null 2>&1; then
  echo "[Mode 1] Using installed nodeunion-agent-launch-tui binary"
  nodeunion-agent-launch-tui
elif command -v cargo >/dev/null 2>&1; then
  if [ -f Cargo.toml ]; then
    echo "[Mode 2] Using cargo to build and run from source"
    cargo run -p nodeunion-agent --bin nodeunion-agent-launch-tui
  else
    echo "nodeunion-agent-launch-tui is not installed and no Cargo.toml was found in the current directory."
    echo "Run this script from the NodeUnion repo root or install the launcher first."
    exit 1
  fi
elif command -v docker >/dev/null 2>&1; then
  echo "[Mode 3] Using Docker container with Rust build environment"
  echo "Building and running agent from Docker container..."
  echo ""
  
  docker run --rm \
    -it \
    -p 8090:8090 \
    rust:1.78-bullseye \
    sh -c '
      apt-get update > /dev/null 2>&1 && \
      apt-get install -y git > /dev/null 2>&1 && \
      git clone --depth 1 https://github.com/NodeUnionClient/nodeunion.git /app && \
      cd /app && \
      cargo run -p nodeunion-agent --bin nodeunion-agent-launch-tui --release
    '
else
  echo "ERROR: No execution method available on this machine."
  echo "Please install one of the following:"
  echo "  1. nodeunion-agent-launch-tui (pre-built binary)"
  echo "  2. Cargo/Rust toolchain"
  echo "  3. Docker"
  exit 1
fi
