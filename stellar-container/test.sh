#!/usr/bin/env bash

# Test script for Stellar NodeUnion Billing Contract

set -e

echo "📦 Building the contract..."
cargo build --target wasm32-unknown-unknown --release

echo ""
echo "✅ Build successful!"
echo ""
echo "📋 Running unit tests..."
cargo test --lib

echo ""
echo "✅ All tests passed!"
echo ""
echo "📊 Contract statistics:"
ls -lh target/wasm32-unknown-unknown/release/nodeunion_billing.wasm || echo "WASM file not found"
echo ""
echo "🎯 Next steps:"
echo "  1. Deploy to Stellar testnet:"
echo "     soroban contract deploy --wasm target/wasm32-unknown-unknown/release/nodeunion_billing.wasm --network testnet"
echo "  2. Run integration tests (requires deployed contract)"
echo "     cargo test --test integration_tests"
