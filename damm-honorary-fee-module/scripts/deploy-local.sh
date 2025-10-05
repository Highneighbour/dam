#!/bin/bash

# Deployment script for DAMM Honorary Fee Module
# Deploys programs to local solana-test-validator

set -e

echo "🚀 Deploying DAMM Honorary Fee Module to local validator..."

# Start local validator in background
echo "🌐 Starting local Solana validator..."
solana-test-validator \
  --reset \
  --bpf-program StreamMock11111111111111111111111111111111 mock_programs/streamflow_mock/target/deploy/streamflow_mock.so \
  --quiet &
VALIDATOR_PID=$!

# Wait for validator to start
echo "⏳ Waiting for validator to initialize..."
sleep 5

# Airdrop SOL to deployer
echo "💰 Airdropping SOL..."
solana airdrop 1000 ~/.config/solana/id.json

# Deploy the DAMM honorary fee program
echo "⚓ Deploying DAMM honorary fee program..."
cd programs/damm_honorary_fee
anchor deploy --provider.cluster localnet

# Deploy the Streamflow mock program
echo "🎭 Deploying Streamflow mock program..."
cd ../../mock_programs/streamflow_mock
anchor deploy --provider.cluster localnet

echo "✅ Deployment complete!"
echo "📋 Program IDs:"
echo "  DAMM Honorary Fee: $(solana program show --programs | grep damm_honorary_fee | awk '{print $1}')"
echo "  Streamflow Mock: $(solana program show --programs | grep streamflow_mock | awk '{print $1}')"

# Keep validator running for tests
echo "🔄 Validator running in background (PID: $VALIDATOR_PID)"
echo "💡 Run 'kill $VALIDATOR_PID' to stop the validator when done"

# Export validator PID for cleanup
echo $VALIDATOR_PID > .validator_pid