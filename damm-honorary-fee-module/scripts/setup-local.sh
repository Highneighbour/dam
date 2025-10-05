#!/bin/bash

# Setup script for DAMM Honorary Fee Module local development
# Installs required toolchain and dependencies in the container

set -e

echo "🚀 Setting up DAMM Honorary Fee Module development environment..."

# Install Rust toolchain
echo "📦 Installing Rust toolchain..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Install Solana CLI
echo "⚡ Installing Solana CLI..."
sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"

# Add Solana to PATH
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Install Anchor CLI
echo "⚓ Installing Anchor CLI..."
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.29.0
avm use 0.29.0

# Install Node.js and npm
echo "📦 Installing Node.js..."
curl -fsSL https://deb.nodesource.com/setup_18.x | bash -
apt-get install -y nodejs

# Generate keypairs for local testing
echo "🔑 Generating local keypairs..."
solana-keygen new -o ~/.config/solana/id.json --no-passphrase
solana-keygen new -o ~/.config/solana/fee-payer.json --no-passphrase
solana-keygen new -o ~/.config/solana/damm-keypair.json --no-passphrase

# Configure Solana for local development
echo "⚙️  Configuring Solana..."
solana config set --url localhost

# Build the programs
echo "🔨 Building programs..."
cd programs/damm_honorary_fee && anchor build
cd ../..

cd mock_programs/streamflow_mock && anchor build
cd ../..

echo "✅ Setup complete! You can now run:"
echo "  ./scripts/deploy-local.sh"
echo "  anchor test"