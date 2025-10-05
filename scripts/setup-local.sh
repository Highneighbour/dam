#!/usr/bin/env bash
set -euo pipefail

# Install toolchains are pre-installed in Dockerfile

# Solana setup
solana-keygen new --no-bip39-passphrase --force -o ~/.config/solana/id.json >/dev/null 2>&1 || true
solana config set --url http://localhost:8899

# Airdrop some SOL
solana airdrop 10 || true

# Node deps for tests
if [ -f package.json ]; then
  yarn install --frozen-lockfile || yarn install
fi

echo "Setup complete."
