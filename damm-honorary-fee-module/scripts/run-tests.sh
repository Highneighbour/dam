#!/bin/bash

# Test runner script for DAMM Honorary Fee Module

set -e

echo "🧪 Running DAMM Honorary Fee Module tests..."

# Check if validator is running
if [ ! -f .validator_pid ] || ! kill -0 $(cat .validator_pid) 2>/dev/null; then
    echo "⚠️  Local validator not running. Starting it..."
    ./scripts/deploy-local.sh
fi

# Run Anchor tests
echo "⚓ Running Anchor tests..."
cd programs/damm_honorary_fee
anchor test -- --nocapture

echo "✅ Tests completed!"