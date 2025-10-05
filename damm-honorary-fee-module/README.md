# DAMM v2 Honorary Quote-Only Fee Position Module

A production-quality Anchor-compatible Rust module implementing DAMM v2 Honorary Quote-Only Fee Position with 24h Distribution Crank.

## Overview

This module provides:

- **Honorary Fee Position Initialization**: Creates quote-only fee positions for DAMM v2 pools
- **24h Permissionless Distribution Crank**: Automated fee distribution with pagination support
- **Quote-Only Enforcement**: Strict validation to prevent base fee accrual
- **Integration Ready**: Works with real cp-amm pools and mock Streamflow for testing

## Architecture

### Key Components

- **Honorary Position**: Program-owned position that only accrues quote fees
- **Policy PDA**: Configuration storage for distribution parameters
- **Progress PDA**: Daily distribution state and pagination tracking
- **Distribution Crank**: Permissionless function for fee claiming and distribution

### PDAs (Program Derived Addresses)

| PDA | Seeds | Purpose |
|-----|-------|---------|
| `InvestorFeePositionOwnerPda` | `["vault", vault_pubkey, "investor_fee_pos_owner"]` | Owns the honorary position |
| `PolicyPda` | `["policy", pool_id]` | Stores distribution configuration |
| `HonoraryPositionAccount` | `["honorary_position", pool_id]` | Position metadata |
| `ProgressPda` | `["progress", policy_id]` | Distribution state tracking |

## Setup & Development

### Prerequisites

- **Rust** (stable toolchain)
- **Anchor CLI** 0.29.0
- **Solana CLI** 1.17.0
- **Node.js** and npm

### Quick Start (Dev Container)

1. **Open in Dev Container** (VS Code):
   ```bash
   # Open folder in container (handles all dependencies)
   code damm-honorary-fee-module
   ```

2. **Manual Setup** (if needed):
   ```bash
   ./scripts/setup-local.sh
   ```

3. **Deploy Locally**:
   ```bash
   ./scripts/deploy-local.sh
   ```

4. **Run Tests**:
   ```bash
   ./scripts/run-tests.sh
   ```

### Manual Setup (Alternative)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.29.0
avm use 0.29.0

# Setup local environment
./scripts/setup-local.sh
./scripts/deploy-local.sh
anchor test
```

## API Reference

### Instructions

#### `initialize_honorary_position`

Initialize a new honorary fee position for a DAMM v2 pool.

**Parameters:**
- `pool_id: Pubkey` - The DAMM v2 pool identifier
- `tick_lower: i32` - Lower tick bound for the position
- `tick_upper: i32` - Upper tick bound for the position
- `vault_pubkey: Pubkey` - Vault public key for PDA seeds
- `investor_fee_share_bps: u16` - Investor fee share in basis points (max 10000)
- `daily_cap_lamports: Option<u64>` - Daily distribution cap (optional)
- `min_payout_lamports: u64` - Minimum payout threshold per investor
- `y0_total_allocation: u64` - Total investor allocation at TGE

**Validation:**
- Validates pool token order to identify quote mint
- Validates tick range for quote-only accrual
- Rejects configurations that may accrue base fees

**Events:**
- `HonoraryPositionInitialized`

#### `crank_distribute_page`

Distribute fees for a page of investors (pagination support).

**Parameters:**
- `page_index: u32` - Current page index for pagination
- `is_final_page_in_day: bool` - Whether this is the last page of the day
- `investor_accounts: Vec<InvestorAccount>` - List of investors in this page

**Behavior:**
- Claims fees from honorary position via cp-amm
- Validates no base fees were accrued
- Reads locked amounts from Streamflow (or mock)
- Calculates pro-rata distribution based on locked amounts
- Distributes quote tokens to investor ATAs
- Updates progress state atomically

**Events:**
- `QuoteFeesClaimed`
- `InvestorPayoutPage`
- `InvestorPayout` (per investor)
- `CreatorPayoutDayClosed` (if final page)

## Error Codes

| Error | Code | Description |
|-------|------|-------------|
| `NotQuoteOnly` | 6000 | Position configuration may accrue base fees |
| `BaseFeesObserved` | 6001 | Base fees detected during claim |
| `DayGateNotOpen` | 6002 | Too early for new day distribution |
| `InvalidPaginationCursor` | 6003 | Page already processed or out of bounds |
| `MinPayoutNotMet` | 6004 | Payout below threshold (carried forward) |
| `StreamflowReadError` | 6005 | Failed to read from Streamflow program |
| `ArithmeticOverflow` | 6006 | Math operation overflow |
| `InvalidTickRange` | 6007 | Invalid tick range specified |

## Integration Guide

### Real cp-amm Integration

1. **Deploy cp-amm**: Deploy the real cp-amm program to local validator
2. **Create Pool**: Set up a DAMM v2 pool with token pairs
3. **Configure Position**: Use the module to create honorary position
4. **Fee Accrual**: Generate fees through trading/swapping
5. **Distribution**: Run crank to distribute accumulated fees

### Streamflow Integration

#### Testing (Mock)
```rust
// Set locked amounts for testing
let streamflow_mock = StreamflowMock::new();
streamflow_mock.set_locked_amount(stream_pubkey, locked_amount).await?;
```

#### Production (Real Streamflow)
Replace mock program ID with real Streamflow program ID:
```rust
// In lib.rs - replace mock program ID
declare_id!("StreamflowProgramIDHere");
```

### Account Structure

#### PolicyAccount
```rust
pub struct PolicyAccount {
    pub pool_id: Pubkey,
    pub vault_pubkey: Pubkey,
    pub creator_wallet: Pubkey,
    pub quote_mint: Pubkey,
    pub investor_fee_share_bps: u16,
    pub daily_cap_lamports: Option<u64>,
    pub min_payout_lamports: u64,
    pub y0_total_allocation: u64,
    pub bump: u8,
}
```

#### ProgressAccount
```rust
pub struct ProgressAccount {
    pub policy_id: Pubkey,
    pub day_id: u64,                    // floor(timestamp / 86400)
    pub last_distribution_ts: i64,
    pub cumulative_distributed_today: u64,
    pub carry_over_lamports: u64,
    pub cursor_idx: u32,
    pub is_closed: bool,
    pub page_payouts: BTreeMap<u32, u64>, // page_index -> total_paid
    pub bump: u8,
}
```

## Testing

### Test Scenarios Covered

1. **Initialization Tests**:
   - ✅ Valid pool/tick config → success
   - ✅ Invalid config → `NotQuoteOnly` error
   - ✅ PDA ownership verification

2. **Distribution Tests**:
   - ✅ Single page distribution
   - ✅ Multi-page pagination
   - ✅ All unlocked scenario (100% to creator)
   - ✅ Dust handling and carry-over
   - ✅ Daily cap enforcement

3. **Edge Cases**:
   - ✅ Base fee detection → failure
   - ✅ Missing investor ATA → creation
   - ✅ Idempotency (re-run same page)
   - ✅ Day gating (24h windows)

### Running Tests

```bash
# Run all tests
./scripts/run-tests.sh

# Run specific test
cargo test test_initialize_honorary_position

# Run with verbose output
RUST_LOG=debug cargo test
```

## Security Considerations

### Quote-Only Enforcement

The module implements strict validation to ensure honorary positions only accrue quote fees:

1. **Preflight Validation**: Tick range analysis during initialization
2. **Runtime Validation**: Base fee detection during claim operations
3. **Atomic Operations**: All-or-nothing distribution to prevent partial states

### Access Control

- **Position Ownership**: Controlled by deterministic PDA, not creator wallet
- **Progress Tracking**: Atomic updates prevent double-spending
- **Pagination Safety**: Cursor validation prevents out-of-order processing

### Failure Modes

- **Network Issues**: Idempotent operations allow safe retries
- **Insufficient Funds**: Graceful handling with carry-over mechanisms
- **Invalid States**: Comprehensive error codes for debugging

## Deployment

### Local Development

```bash
# Deploy programs to local validator
./scripts/deploy-local.sh

# Program IDs:
# - DAMM Honorary Fee: Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFp1J6
# - Streamflow Mock: StreamMock11111111111111111111111111111111
```

### Production Deployment

1. **Update Program IDs**: Replace test IDs with production addresses
2. **Deploy cp-amm**: Deploy real cp-amm program
3. **Configure Streamflow**: Update to use real Streamflow program ID
4. **Set Up Oracles**: Configure price feeds for quote mint identification

## Contributing

1. **Development Setup**: Use dev container or run `./scripts/setup-local.sh`
2. **Code Style**: Follow Anchor best practices, use `cargo fmt` and `cargo clippy`
3. **Testing**: Add tests for new functionality, ensure all tests pass
4. **Documentation**: Update README and inline comments for API changes

## License

MIT License - see LICENSE file for details.

## Support

For integration questions or issues:
- Review the integration examples in `/tests/`
- Check error codes and event logs for debugging
- Ensure cp-amm and Streamflow program IDs match your deployment