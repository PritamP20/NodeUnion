# NodeUnion Solana Billing Layer

This folder contains the Anchor contract workspace for on-chain billing.

## Program

- Program: nodeunion_billing
- Instructions:
  - initialize_config(authority, token_mint, treasury, rate_per_unit)
  - open_escrow(job_id, network_id, provider, max_units, deposit_amount)
  - record_usage(units)
  - close_escrow()

## What this gives you

- Every job can be represented by an on-chain escrow account.
- Provider payout is transferred during usage metering.
- User gets automatic refund of unspent escrow balance on close.
- Billing events are auditable from chain logs.

## Mapping from orchestrator lifecycle

1) When user submits job:
- Validate requester wallet.
- Estimate deposit amount from requested resources.
- Call open_escrow.
- Store escrow PDA and tx signature in DB.

2) While job is running:
- Aggregate usage every billing window.
- Call record_usage(units) from orchestrator signer.
- Store each tx signature as settlement event.

3) When job completes/fails/stops:
- Call close_escrow.
- Persist final settlement tx hash and final amounts.

## Local development commands

Prerequisites:
- Solana CLI
- Anchor CLI

From this folder:

- Build program: anchor build
- Run local validator tests: anchor test
- Deploy to localnet: anchor deploy

## Production notes

- Keep orchestrator payer key in a secure secret manager.
- Use priority fees and retries for record_usage.
- Add idempotency keys in DB to avoid duplicate settlement writes.
- Use devnet first, then migrate to mainnet-beta after soak testing.


zEmSJV8TWmSwHX2f6RdyFvZgvCwQaJ9ZrLfdQtidexo