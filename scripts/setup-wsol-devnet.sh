#!/usr/bin/env bash
set -euo pipefail

# setup-wsol-devnet.sh
# Create WSOL associated token accounts for user, provider, and escrow on devnet
# and optionally wrap SOL into the user's WSOL ATA. Adjust env vars as needed.

RPC=${SOLANA_RPC_URL:-https://api.devnet.solana.com}
PAYER_KEYPAIR=${SOLANA_PAYER_KEYPAIR:-$HOME/.config/solana/id.json}
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="$PROJECT_ROOT/.env.wsol"

WSOL_MINT="So11111111111111111111111111111111111111112"

usage() {
  cat <<EOF
Usage: $0 --user-wallet <USER_PUBKEY> --provider-wallet <PROVIDER_PUBKEY> [--wrap-amount <LAMPORTS_OR_SOL>]

Creates WSOL associated token accounts and prints environment variables to export.
If your `spl-token` has the `wrap` subcommand, the script will attempt to wrap the requested amount
into the created user WSOL account. Amount can be a decimal SOL (e.g. 1 or 0.5).

Example:
  $0 --user-wallet <USER_PUBKEY> --provider-wallet <PROVIDER_PUBKEY> --wrap-amount 1

EOF
  exit 1
}

resolve_or_create_ata() {
  local owner="$1"
  local label="$2"

  local existing
  existing=$(spl-token accounts "$WSOL_MINT" --owner "$owner" --addresses-only --fee-payer "$PAYER_KEYPAIR" --url "$RPC" 2>/dev/null | head -1 || true)
  if [[ -n "$existing" ]]; then
    echo "$existing"
    return 0
  fi

  local created
  created=$(spl-token create-account "$WSOL_MINT" --owner "$owner" --fee-payer "$PAYER_KEYPAIR" --url "$RPC" 2>&1)
  local parsed
  parsed=$(echo "$created" | grep -oE "[1-9A-HJ-NP-Za-km-z]{32,64}" | head -1 || true)
  if [[ -z "$parsed" ]]; then
    echo "Failed to resolve or create $label WSOL ATA:" >&2
    echo "$created" >&2
    exit 3
  fi

  echo "$parsed"
}

if ! command -v solana >/dev/null 2>&1; then
  echo "solana CLI not found. Install from https://docs.solana.com/cli/install" >&2
  exit 2
fi
if ! command -v spl-token >/dev/null 2>&1; then
  echo "spl-token CLI not found. Install via cargo or package manager." >&2
  exit 2
fi

USER_WALLET=""
PROVIDER_WALLET=""
WRAP_AMOUNT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --user-wallet) USER_WALLET="$2"; shift 2;;
    --provider-wallet) PROVIDER_WALLET="$2"; shift 2;;
    --wrap-amount) WRAP_AMOUNT="$2"; shift 2;;
    -h|--help) usage;;
    *) echo "Unknown arg: $1"; usage;;
  esac
done

if [[ -z "$USER_WALLET" || -z "$PROVIDER_WALLET" ]]; then
  echo "--user-wallet and --provider-wallet are required." >&2
  usage
fi

echo "Using RPC: $RPC"
echo "Using payer keypair: $PAYER_KEYPAIR"

echo "Creating WSOL associated token account for USER ($USER_WALLET)..."
USER_ATA=$(resolve_or_create_ata "$USER_WALLET" "user")

echo "Created USER_ATA=$USER_ATA"

echo "Creating WSOL associated token account for PROVIDER ($PROVIDER_WALLET)..."
PROVIDER_ATA=$(resolve_or_create_ata "$PROVIDER_WALLET" "provider")

echo "Created PROVIDER_ATA=$PROVIDER_ATA"

# Create an escrow token account owned by the escrow PDA will be created on-chain by the program,
# but we still create an ATA placeholder (this may not be needed depending on program behavior)
ESCROW_ATA=$(spl-token create-account "$WSOL_MINT" --fee-payer "$PAYER_KEYPAIR" --url "$RPC" 2>&1 || true)
ESCROW_ATA_RAW="$ESCROW_ATA"
ESCROW_ATA=$(echo "$ESCROW_ATA_RAW" | grep -oE "[1-9A-HJ-NP-Za-km-z]{32,64}" | head -1 || true)
if [[ -z "$ESCROW_ATA" ]]; then
  echo "Failed to parse escrow ATA from spl-token output:" >&2
  echo "$ESCROW_ATA_RAW" >&2
  exit 5
fi

echo "Created ESCROW_ATA (placeholder)=$ESCROW_ATA"

# Optionally fund the user's WSOL ATA with native SOL and sync it.
if [[ -n "$WRAP_AMOUNT" ]]; then
  echo "Funding $USER_ATA with $WRAP_AMOUNT SOL and syncing native balance..."
  if solana transfer "$USER_ATA" "$WRAP_AMOUNT" --allow-unfunded-recipient --from "$PAYER_KEYPAIR" --url "$RPC" >/dev/null 2>&1; then
    if spl-token sync-native --address "$USER_ATA" --fee-payer "$PAYER_KEYPAIR" --url "$RPC" >/dev/null 2>&1; then
      echo "Funded and synced $USER_ATA with $WRAP_AMOUNT SOL"
    else
      echo "Transferred SOL to $USER_ATA, but sync-native failed. Run this manually:" >&2
      echo "  spl-token sync-native --address \"$USER_ATA\" --fee-payer \"$PAYER_KEYPAIR\" --url \"$RPC\"" >&2
    fi
  else
    echo "Failed to transfer SOL into $USER_ATA. To do it manually:" >&2
    echo "  solana transfer \"$USER_ATA\" \"$WRAP_AMOUNT\" --allow-unfunded-recipient --from \"$PAYER_KEYPAIR\" --url \"$RPC\"" >&2
    echo "  spl-token sync-native --address \"$USER_ATA\" --fee-payer \"$PAYER_KEYPAIR\" --url \"$RPC\"" >&2
  fi
fi

cat <<EOF

Environment variables to export for testing (copy these into your shell):

export SOLANA_RPC_URL="$RPC"
export SOLANA_PAYER_KEYPAIR="$PAYER_KEYPAIR"
export SOLANA_TOKEN_MINT="$WSOL_MINT"
export SOLANA_USER_TOKEN_ACCOUNT="$USER_ATA"
export SOLANA_PROVIDER_TOKEN_ACCOUNT="$PROVIDER_ATA"
export SOLANA_ESCROW_TOKEN_ACCOUNT="$ESCROW_ATA"

NOTE: The program expects the escrow token account to be owned by the escrow PDA; if the Anchor
program creates/initializes the escrow token account itself, you may not need to pre-create the
placeholder escrow ATA. Check program logs if CPI fails with InvalidTokenAccount.

EOF

cat > "$ENV_FILE" <<EOF
export SOLANA_RPC_URL="$RPC"
export SOLANA_PAYER_KEYPAIR="$PAYER_KEYPAIR"
export SOLANA_PROGRAM_ID="zEmSJV8TWmSwHX2f6RdyFvZgvCwQaJ9ZrLfdQtidexo"
export SOLANA_TOKEN_MINT="$WSOL_MINT"
export SOLANA_USER_TOKEN_ACCOUNT="$USER_ATA"
export SOLANA_PROVIDER_TOKEN_ACCOUNT="$PROVIDER_ATA"
export SOLANA_ESCROW_TOKEN_ACCOUNT="$ESCROW_ATA"
EOF

echo "Wrote reusable env file: $ENV_FILE"
echo "Load it with: source \"$ENV_FILE\""

echo "Done."
