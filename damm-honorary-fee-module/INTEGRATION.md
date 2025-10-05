# Integration Guide: DAMM v2 Honorary Fee Module

This document explains how to integrate the DAMM v2 Honorary Quote-Only Fee Position Module into your launch protocol.

## Overview

The module provides two main functions:
1. **Initialize Honorary Position**: Set up a quote-only fee position for your DAMM v2 pool
2. **Distribution Crank**: Permissionless function to distribute accumulated fees to investors

## Step 1: Program Deployment

### Deploy the Module

```bash
# Deploy to your target environment (devnet/mainnet)
anchor deploy --provider.cluster <your-cluster>
```

### Deploy cp-amm (if not already deployed)

The module requires a real cp-amm program for fee claiming. Deploy cp-amm to your local validator or use an existing deployment.

```bash
# Example for local deployment
solana program deploy /path/to/cp-amm.so --program-id <cp-amm-program-id>
```

### Deploy Mock Streamflow (for testing)

For local testing, deploy the included mock Streamflow program:

```bash
cd mock_programs/streamflow_mock
anchor deploy --provider.cluster localnet
```

## Step 2: Initialize Honorary Position

### Parameters Required

```rust
let pool_id = Pubkey::from_str("YourPoolAddress"); // Your DAMM v2 pool
let tick_lower = -100000; // Lower tick bound (quote-only range)
let tick_upper = 100000;  // Upper tick bound (quote-only range)
let vault_pubkey = Pubkey::from_str("YourVaultAddress"); // For PDA seeds
let investor_fee_share_bps = 8000; // 80% to investors, 20% to creator
let daily_cap_lamports = Some(1_000_000_000_000); // 1000 tokens daily cap
let min_payout_lamports = 100_000_000; // 0.1 tokens minimum payout
let y0_total_allocation = 1_000_000_000_000_000; // Total allocation at TGE
```

### Call Initialization

```rust
use damm_honorary_fee::damm_honorary_fee::*;

let accounts = InitializeHonoraryPosition {
    pool: pool_account,
    token_mint_0: quote_mint_account, // Must be quote mint
    token_mint_1: base_mint_account,
    position: position_account,
    position_nft_mint: nft_mint_account,
    investor_fee_position_owner_pda: pda_account,
    vault_pubkey: vault_account,
    creator_wallet: creator_wallet_account,
    policy_pda: policy_pda_account,
    honorary_position: honorary_position_account,
    program_quote_treasury_ata: treasury_ata_account,
    quote_mint: quote_mint_account,
    token_program: token_program_account,
    associated_token_program: ata_program_account,
    system_program: system_program_account,
};

let cpi_ctx = CpiContext::new(cpi_program, accounts);

initialize_honorary_position(
    cpi_ctx,
    pool_id,
    tick_lower,
    tick_upper,
    vault_pubkey,
    investor_fee_share_bps,
    daily_cap_lamports,
    min_payout_lamports,
    y0_total_allocation,
)?;
```

## Step 3: Set Up Investor Data

### For Each Distribution

Prepare investor data including:
- Streamflow stream pubkey for each investor
- Investor's quote token ATA
- Expected locked amounts (from Streamflow)

```rust
struct InvestorData {
    stream_pubkey: Pubkey,
    investor_quote_ata: Pubkey,
    expected_locked_amount: u64,
}
```

### Pagination Strategy

Split investors into pages (recommended: 50-100 per page):

```rust
const PAGE_SIZE: usize = 50;

let investor_pages: Vec<Vec<InvestorData>> = investors
    .chunks(PAGE_SIZE)
    .map(|chunk| chunk.to_vec())
    .collect();
```

## Step 4: Run Distribution Crank

### For Each Page

```rust
for (page_index, investors) in investor_pages.iter().enumerate() {
    let is_final_page = page_index == investor_pages.len() - 1;

    // Prepare investor accounts for this page
    let investor_accounts = investors
        .iter()
        .map(|inv| InvestorAccount {
            investor_quote_ata: inv.investor_quote_ata,
            stream_pubkey: inv.stream_pubkey,
            locked_amount: query_locked_amount(inv.stream_pubkey), // From Streamflow
        })
        .collect();

    // Call the crank
    let accounts = CrankDistributePage {
        policy_pda: policy_pda_account,
        honorary_position: honorary_position_account,
        progress_pda: progress_pda_account,
        program_quote_treasury_ata: treasury_ata_account,
        investor_fee_position_owner_pda: pda_account,
        vault_pubkey: vault_account,
        quote_mint: quote_mint_account,
        token_program: token_program_account,
    };

    let cpi_ctx = CpiContext::new(cpi_program, accounts);

    crank_distribute_page(
        cpi_ctx,
        page_index as u32,
        is_final_page,
        investor_accounts,
    )?;
}
```

## Step 5: Production Streamflow Integration

### Replace Mock with Real Streamflow

1. **Update Program ID**:
   ```rust
   // In lib.rs
   declare_id!("STREAMFLOW_PROGRAM_ID"); // Replace mock ID
   ```

2. **Update Query Logic**:
   ```rust
   // Implement real Streamflow locked amount query
   fn query_locked_amount(stream_pubkey: Pubkey) -> Result<u64> {
       // Call real Streamflow program instruction
       let accounts = GetLockedAmount {
           locked_account: locked_account_for_stream(stream_pubkey),
           stream_pubkey: stream_pubkey,
       };

       let instruction = Instruction {
           program_id: STREAMFLOW_PROGRAM_ID,
           accounts: accounts.to_account_metas(None),
           data: instruction_data, // get_locked_amount discriminator
       };

       // Execute instruction and parse result
       // Return locked amount
   }
   ```

3. **Deploy Updated Program**:
   ```bash
   anchor build
   anchor deploy --provider.cluster mainnet
   ```

## Step 6: Monitoring and Maintenance

### Event Monitoring

Subscribe to emitted events for tracking:

```rust
// Listen for events
let events = ProgramAccount::<Event>::accounts(cpi_ctx.accounts.program)?;

for event in events {
    match event {
        Event::HonoraryPositionInitialized { pool_id, .. } => {
            println!("Position initialized for pool: {}", pool_id);
        }
        Event::InvestorPayout { investor_quote_ata, amount, .. } => {
            println!("Paid {} to investor {}", amount, investor_quote_ata);
        }
        Event::CreatorPayoutDayClosed { day_id, remainder_amount, .. } => {
            println!("Day {} closed, creator remainder: {}", day_id, remainder_amount);
        }
        _ => {}
    }
}
```

### Health Checks

1. **Daily Distribution Check**:
   ```rust
   let progress = ProgramAccount::<ProgressAccount>::try_from(progress_pda)?;
   let current_day = get_current_day();

   if progress.day_id < current_day && !progress.is_closed {
       // Alert: Distribution may be stuck
   }
   ```

2. **Fee Accrual Monitoring**:
   ```rust
   let position_fees = query_position_fees(honorary_position_account)?;
   if position_fees.quote_amount > threshold {
       // Trigger distribution
   }
   ```

## Error Handling

### Common Integration Errors

1. **BaseFeesObserved**:
   - Check tick range configuration
   - Verify pool token ordering
   - Ensure position only accrues quote fees

2. **DayGateNotOpen**:
   - Wait for 24h window
   - Check system clock synchronization

3. **InvalidPaginationCursor**:
   - Process pages in order
   - Don't skip pages
   - Handle failed transactions properly

### Retry Logic

```rust
let max_retries = 3;
let mut attempts = 0;

loop {
    match crank_distribute_page(cpi_ctx, page_index, is_final, investors) {
        Ok(_) => break,
        Err(error) => {
            attempts += 1;
            if attempts >= max_retries {
                return Err(error);
            }

            if matches!(error, DammHonoraryFeeError::InvalidPaginationCursor) {
                // Check progress PDA and adjust page_index
                let progress = get_progress_account()?;
                page_index = progress.cursor_idx;
            }

            sleep(Duration::from_secs(1));
        }
    }
}
```

## Performance Considerations

### Gas Optimization

1. **Page Size**: Optimal 50-100 investors per page
2. **Batch Processing**: Group related operations
3. **ATA Creation**: Only create when necessary (policy setting)

### Scaling

1. **Multiple Pools**: Deploy separate module instances per pool
2. **Parallel Processing**: Run cranks for different pools simultaneously
3. **Off-chain Coordination**: Use off-chain service for pagination orchestration

## Security Integration

### Access Control

- **Position Ownership**: Verify PDA ownership before operations
- **Creator Wallet**: Ensure only authorized wallet can initialize
- **Progress Updates**: Validate atomic progress updates

### Audit Trail

- **Event Logging**: Monitor all distribution events
- **Balance Verification**: Cross-check distributed amounts
- **Progress Consistency**: Ensure progress PDA state consistency

## Troubleshooting

### Common Issues

1. **"NotQuoteOnly" Error**:
   - Review tick range calculation
   - Check pool price vs tick range
   - Consult cp-amm documentation for quote-only validation

2. **"BaseFeesObserved" Error**:
   - Position accrued base fees unexpectedly
   - Check for pool rebalancing or extreme price movements
   - Consider adjusting tick range

3. **Distribution Failures**:
   - Check investor ATA creation permissions
   - Verify sufficient treasury balance
   - Monitor for network congestion

### Debug Commands

```bash
# Check program logs
solana logs --program <program-id>

# Query account state
solana account <account-address>

# Check transaction details
solana transaction-history <signature>
```

## Support and Maintenance

### Version Updates

- Monitor for module updates and security patches
- Test updates on devnet before mainnet deployment
- Maintain compatibility with cp-amm and Streamflow versions

### Emergency Procedures

1. **Pause Distribution**:
   ```rust
   // Update policy to disable distributions temporarily
   let policy = get_policy_account()?;
   // Set investor_fee_share_bps = 0
   ```

2. **Emergency Fund Recovery**:
   ```rust
   // Only if critical - withdraw from treasury ATA
   // Requires careful review and testing
   ```

For additional support, refer to the test implementations in `/tests/` and the comprehensive error documentation in the main README.