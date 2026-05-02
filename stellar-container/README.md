# NodeUnion Stellar Billing Smart Contract

## Status

✅ **Compiled & Tested**  
🟢 **Ready for Deployment**

This is a Soroban smart contract implementation of the NodeUnion billing layer for the Stellar network. The contract was successfully converted from Solana Anchor and compiles to optimized WASM (5.5 KB).

## Quick Start

### Build
```bash
cargo build --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/nodeunion_billing.wasm`

### Deploy to Testnet
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/nodeunion_billing.wasm \
  --network testnet \
  --source ~/.config/stellar/testnet-key.json
```

## Contract Overview

This contract manages on-chain billing for node providers running jobs on NodeUnion. It maintains:

- **BillingConfig**: Global configuration (authority, token, treasury, rate)
- **JobEscrow**: Per-job billing account with status tracking  
- **EscrowStatus**: Open/Closed enum for lifecycle management

## Functions

### Configuration
- `initialize_config(authority, token, treasury, rate_per_unit)` - Setup billing
- `get_config()` - Retrieve global config

### Escrow Management
- `open_escrow(job_id, max_units, deposit_amount, provider_wallet)` - Create job escrow
- `get_escrow(job_id)` - Retrieve escrow state
- `record_usage(job_id, units)` - Track usage and charge provider
- `close_escrow(job_id)` - Finalize and refund overpayment

## Stellar Integration

### Key Differences from Solana
| Aspect | Solana (Anchor) | Stellar (Soroban) |
|--------|-----------------|-------------------|
| Storage | Program accounts | Persistent KV store |
| Auth | Signer constraints | `require_auth()` |
| Tokens | SPL Token Program | Stellar assets |
| Error Model | Error codes | Panic-based |

### Data Model  
- Job IDs: `u64` (numeric, simpler than Strings)
- Storage: Direct key-value persistence
- Events: Soroban event system (publishable)

## Testing

```bash
# Compile-time tests
cargo test --lib

# After deployment, run integration tests  
cargo test --test integration_tests
```

## Build Information

- **Rust:** 1.85.0+
- **Soroban SDK:** 20.5.0
- **Compiled:** May 1, 2026
- **Size:** 5.5 KB (optimized WASM)
- **Warnings:** 0 errors, 6 non-critical warnings

See [BUILD_REPORT.md](BUILD_REPORT.md) for detailed build information.
