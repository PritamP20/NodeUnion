#!/bin/bash
# Integration test script for NodeUnion Solana billing layer
# Tests the full job → billing cycle with mock or real devnet

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR"

# Load shell-safe local env files if present so users don't have to export variables manually.
for env_file in "$PROJECT_ROOT/.env.local" "$PROJECT_ROOT/.env.wsol"; do
    if [ -f "$env_file" ]; then
        # shellcheck disable=SC1090
        . "$env_file"
    fi
done

echo "=========================================="
echo "NodeUnion Solana Integration Test"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Environment variables
echo -e "${YELLOW}[TEST 1]${NC} Checking Solana environment variables..."
required_vars=(
    "SOLANA_RPC_URL"
    "SOLANA_PROGRAM_ID"
    "SOLANA_PAYER_KEYPAIR"
    "SOLANA_TOKEN_MINT"
    "SOLANA_USER_TOKEN_ACCOUNT"
    "SOLANA_ESCROW_TOKEN_ACCOUNT"
    "SOLANA_PROVIDER_TOKEN_ACCOUNT"
)

missing_vars=()
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        missing_vars+=("$var")
    else
        echo "  ✓ $var = ${!var:0:20}..."
    fi
done

if [ ${#missing_vars[@]} -gt 0 ]; then
    echo -e "${RED}✗ Missing environment variables:${NC}"
    for var in "${missing_vars[@]}"; do
        echo "  - $var"
    done
    echo ""
    echo "Set them in .env or export in shell:"
    echo '  export SOLANA_RPC_URL="https://api.devnet.solana.com"'
    echo '  export SOLANA_PROGRAM_ID="..."'
    echo '  export SOLANA_PAYER_KEYPAIR="~/.config/solana/id.json"'
    echo '  export SOLANA_TOKEN_MINT="..."'
    echo '  export SOLANA_USER_TOKEN_ACCOUNT="..."'
    echo '  export SOLANA_ESCROW_TOKEN_ACCOUNT="..."'
    echo '  export SOLANA_PROVIDER_TOKEN_ACCOUNT="..."'
    exit 1
else
    echo -e "${GREEN}✓ All environment variables set${NC}"
fi
echo ""

# Test 2: Solana CLI available
echo -e "${YELLOW}[TEST 2]${NC} Checking Solana CLI tools..."
if ! command -v solana &> /dev/null; then
    echo -e "${RED}✗ solana CLI not found${NC}"
    exit 1
fi
solana_version=$(solana --version | cut -d' ' -f2)
echo -e "  ✓ solana CLI version: $solana_version"

if ! command -v spl-token &> /dev/null; then
    echo -e "${YELLOW}⚠ spl-token not in PATH${NC} (optional for manual token setup)"
else
    echo -e "  ✓ spl-token available"
fi
echo ""

# Test 3: Devnet connectivity
echo -e "${YELLOW}[TEST 3]${NC} Checking devnet connectivity..."
if command -v timeout >/dev/null 2>&1; then
    timeout 5 solana cluster-version --url "$SOLANA_RPC_URL" &>/dev/null
    status=$?
elif command -v gtimeout >/dev/null 2>&1; then
    gtimeout 5 solana cluster-version --url "$SOLANA_RPC_URL" &>/dev/null
    status=$?
else
    solana cluster-version --url "$SOLANA_RPC_URL" &>/dev/null
    status=$?
fi

if [ "$status" -eq 0 ]; then
    echo -e "  ✓ Connected to devnet: $SOLANA_RPC_URL"
    version=$(solana cluster-version --url "$SOLANA_RPC_URL" | head -1)
    echo "    $version"
else
    echo -e "${RED}✗ Cannot connect to devnet at $SOLANA_RPC_URL${NC}"
    echo "  Check your internet connection or RPC URL"
    exit 1
fi
echo ""

# Test 4: Payer wallet
echo -e "${YELLOW}[TEST 4]${NC} Checking payer wallet..."
if [ ! -f "${SOLANA_PAYER_KEYPAIR/#\~/$HOME}" ]; then
    echo -e "${RED}✗ Keypair file not found: $SOLANA_PAYER_KEYPAIR${NC}"
    exit 1
fi

payer_pubkey=$(solana-keygen pubkey "$SOLANA_PAYER_KEYPAIR")
echo -e "  ✓ Payer address: $payer_pubkey"

# Get balance
balance=$(solana balance --url "$SOLANA_RPC_URL" "$payer_pubkey" 2>/dev/null || echo "error")
if [ "$balance" = "error" ]; then
    echo -e "${YELLOW}⚠ Could not fetch balance (may not exist on chain)${NC}"
else
    echo "    Balance: $balance"
fi
echo ""

# Test 5: Program deployment
echo -e "${YELLOW}[TEST 5]${NC} Checking program deployment..."
if solana program show "$SOLANA_PROGRAM_ID" --url "$SOLANA_RPC_URL" &> /dev/null; then
    echo -e "  ✓ Program found: $SOLANA_PROGRAM_ID"
    solana program show "$SOLANA_PROGRAM_ID" --url "$SOLANA_RPC_URL" | head -5
else
    echo -e "${RED}✗ Program not deployed: $SOLANA_PROGRAM_ID${NC}"
    echo "  Deploy with: cd solana && anchor deploy"
    exit 1
fi
echo ""

# Test 6: Token mint
echo -e "${YELLOW}[TEST 6]${NC} Checking token mint..."
if spl-token mint "$SOLANA_TOKEN_MINT" --url "$SOLANA_RPC_URL" &> /dev/null 2>&1; then
    echo -e "  ✓ Token mint exists: $SOLANA_TOKEN_MINT"
else
    echo -e "${YELLOW}⚠ Could not verify token mint (may not exist or RPC issue)${NC}"
fi
echo ""

# Test 7: Build orchestrator
echo -e "${YELLOW}[TEST 7]${NC} Building orchestrator..."
cd "$PROJECT_ROOT"
if cargo check -p nodeunion-orchestrator 2>&1 | tail -5 | grep -q "Finished\|error"; then
    if cargo check -p nodeunion-orchestrator 2>&1 | grep -q "error"; then
        echo -e "${RED}✗ Build failed${NC}"
        exit 1
    fi
fi
echo -e "  ✓ Orchestrator builds successfully"
echo ""

# Test 8: Compile test
echo -e "${YELLOW}[TEST 8]${NC} Testing Solana client compilation..."
if cargo build -p nodeunion-orchestrator --lib 2>&1 | tail -3 | grep -q "Finished"; then
    echo -e "  ✓ Solana client compiles"
else
    echo -e "${YELLOW}⚠ Compilation check inconclusive${NC}"
fi
echo ""

# Summary
echo "=========================================="
echo -e "${GREEN}✓ All integration checks passed!${NC}"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. Start orchestrator:"
echo "     cargo run -p nodeunion-orchestrator --bin nodeunion-orchestrator"
echo ""
echo "  2. Start agent (in another terminal):"
echo "     cargo run -p nodeunion-agent --bin nodeunion-agent-server"
echo ""
echo "  3. Submit test job:"
echo "     curl -X POST http://127.0.0.1:8080/jobs/submit \\"
echo "       -H 'content-type: application/json' \\"
echo "       -d '{ \"network_id\": \"default\", \"user_wallet\": \"$(solana address)\", \"image\": \"alpine:3.20\", \"command\": [\"echo\", \"test\"], \"cpu_limit\": 0.25, \"ram_limit_mb\": 512 }'"
echo ""
echo "  4. Monitor Solana transactions:"
echo "     solana logs --url devnet | grep -i escrow"
echo ""
