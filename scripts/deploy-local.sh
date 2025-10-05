#!/usr/bin/env bash
set -euo pipefail

# Start validator in background if not running
if ! pgrep -x "solana-test-val" >/dev/null && ! pgrep -x "solana-test-validator" >/dev/null; then
  echo "Starting local validator..."
  solana-test-validator --reset --rpc-port 8899 --quiet > /tmp/validator.log 2>&1 &
  sleep 5
fi

anchor build

# Anchor will deploy in tests; here we only build to ensure artifacts exist

echo "Build complete."
