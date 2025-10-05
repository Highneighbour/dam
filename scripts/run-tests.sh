#!/usr/bin/env bash
set -euo pipefail

# Ensure validator is running
if ! pgrep -x "solana-test-val" >/dev/null && ! pgrep -x "solana-test-validator" >/dev/null; then
  solana-test-validator --reset --rpc-port 8899 --quiet > /tmp/validator.log 2>&1 &
  sleep 5
fi

# Run anchor ts tests if present, else run cargo tests
if compgen -G "tests/**/*.ts" > /dev/null || [ -d tests ]; then
  anchor test -- --nocapture
else
  cargo test --workspace --all-features -- --nocapture
fi
