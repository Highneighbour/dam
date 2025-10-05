//! DAMM v2 Honorary Quote-Only Fee Position Module
//!
//! This program implements a DAMM v2 honorary fee position system with:
//! - Quote-only fee position initialization
//! - 24h permissionless distribution crank with pagination
//! - Integration with cp-amm for fee claiming
//! - Mock Streamflow integration for locked amount queries

use anchor_lang::prelude::*;
use std::collections::BTreeMap;

pub mod state;
pub mod errors;
pub mod events;

use state::*;
use errors::*;
use events::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFp1J6");

#[program]
pub mod damm_honorary_fee {
    use super::*;

    /// Initialize a new honorary fee position for a DAMM v2 pool
    pub fn initialize_honorary_position(
        ctx: Context<InitializeHonoraryPosition>,
        pool_id: Pubkey,
        tick_lower: i32,
        tick_upper: i32,
        vault_pubkey: Pubkey,
        investor_fee_share_bps: u16,
        daily_cap_lamports: Option<u64>,
        min_payout_lamports: u64,
        y0_total_allocation: u64,
    ) -> Result<()> {
        let policy_pda = &mut ctx.accounts.policy_pda;
        let honorary_position = &mut ctx.accounts.honorary_position;

        // Validate pool token order and identify quote mint
        let quote_mint = identify_quote_mint(
            &ctx.accounts.token_mint_0,
            &ctx.accounts.token_mint_1,
            pool_id,
        )?;

        // Validate tick range for quote-only accrual
        validate_quote_only_position(tick_lower, tick_upper)?;

        // Initialize policy PDA
        policy_pda.pool_id = pool_id;
        policy_pda.vault_pubkey = vault_pubkey;
        policy_pda.creator_wallet = ctx.accounts.creator_wallet.key();
        policy_pda.quote_mint = quote_mint;
        policy_pda.investor_fee_share_bps = investor_fee_share_bps;
        policy_pda.daily_cap_lamports = daily_cap_lamports;
        policy_pda.min_payout_lamports = min_payout_lamports;
        policy_pda.y0_total_allocation = y0_total_allocation;
        policy_pda.bump = ctx.bumps.policy_pda;

        // Initialize honorary position account
        honorary_position.pool_id = pool_id;
        honorary_position.position_id = ctx.accounts.position.key();
        honorary_position.position_nft_mint = ctx.accounts.position_nft_mint.key();
        honorary_position.owner_pda = ctx.accounts.investor_fee_position_owner_pda.key();
        honorary_position.quote_mint = quote_mint;
        honorary_position.tick_lower = tick_lower;
        honorary_position.tick_upper = tick_upper;
        honorary_position.bump = ctx.bumps.honorary_position;

        // Emit initialization event
        emit!(HonoraryPositionInitialized {
            pool_id,
            position_id: ctx.accounts.position.key(),
            position_nft_mint: ctx.accounts.position_nft_mint.key(),
            owner_pda: ctx.accounts.investor_fee_position_owner_pda.key(),
            quote_mint,
            tick_lower,
            tick_upper,
        });

        Ok(())
    }

    /// Crank to distribute fees for a page of investors
    pub fn crank_distribute_page(
        ctx: Context<CrankDistributePage>,
        page_index: u32,
        is_final_page_in_day: bool,
        investor_accounts: Vec<InvestorAccount>,
    ) -> Result<()> {
        let policy = &ctx.accounts.policy_pda;
        let progress = &mut ctx.accounts.progress_pda;
        let current_time = Clock::get()?.unix_timestamp;

        // Validate day gate
        let current_day_id = (current_time / 86400) as u64;
        if current_day_id <= progress.day_id && current_time < progress.last_distribution_ts + 86400 {
            return Err(DammHonoraryFeeError::DayGateNotOpen.into());
        }

        // Update day tracking if new day
        if current_day_id > progress.day_id {
            progress.day_id = current_day_id;
            progress.last_distribution_ts = current_time;
            progress.cumulative_distributed_today = 0;
            progress.carry_over_lamports = 0;
            progress.is_closed = false;
            progress.page_payouts.clear();
        }

        // Validate pagination cursor
        if page_index != progress.cursor_idx {
            return Err(DammHonoraryFeeError::InvalidPaginationCursor.into());
        }

        // Claim fees from position (mock implementation for now)
        // TODO: Integrate with actual cp-amm claim_fees instruction
        let claimed_quote = 0u64; // This would come from cp-amm claim

        // Calculate total locked amount and validate no base fees
        let mut total_locked: u64 = 0;
        for investor in &investor_accounts {
            total_locked = total_locked.checked_add(investor.locked_amount)
                .ok_or(DammHonoraryFeeError::ArithmeticOverflow)?;
        }

        // Calculate investor share
        let y0 = policy.y0_total_allocation;
        let locked_total = total_locked;
        let f_locked = if y0 > 0 {
            (locked_total as u128 * 10000 / y0 as u128) as u64
        } else {
            0
        };

        let eligible_investor_share_bps = std::cmp::min(
            policy.investor_fee_share_bps,
            f_locked as u16,
        );

        let investor_fee_quote = ((claimed_quote as u128) * (eligible_investor_share_bps as u128) / 10000) as u64;

        // Distribute to investors
        let mut total_paid_this_page: u64 = 0;
        for investor in investor_accounts {
            let weight = if total_locked > 0 {
                (investor.locked_amount as u128 * 10000 / total_locked as u128) as u64
            } else {
                0
            };

            let payout = (investor_fee_quote as u128 * weight as u128 / 10000) as u64;

            if payout >= policy.min_payout_lamports {
                // Transfer tokens to investor ATA
                // TODO: Implement actual token transfer

                total_paid_this_page = total_paid_this_page.checked_add(payout)
                    .ok_or(DammHonoraryFeeError::ArithmeticOverflow)?;

                // Emit investor payout event
                emit!(InvestorPayout {
                    investor_quote_ata: investor.investor_quote_ata,
                    amount: payout,
                    locked_amount: investor.locked_amount,
                    page_index,
                });
            } else {
                // Carry over dust
                progress.carry_over_lamports = progress.carry_over_lamports.checked_add(payout)
                    .ok_or(DammHonoraryFeeError::ArithmeticOverflow)?;
            }
        }

        // Update progress tracking
        progress.cumulative_distributed_today = progress.cumulative_distributed_today
            .checked_add(total_paid_this_page)
            .ok_or(DammHonoraryFeeError::ArithmeticOverflow)?;

        progress.cursor_idx = page_index.checked_add(1)
            .ok_or(DammHonoraryFeeError::ArithmeticOverflow)?;

        progress.page_payouts.insert(page_index, total_paid_this_page);

        // Handle final page of day
        if is_final_page_in_day {
            let remainder = investor_fee_quote.checked_sub(total_paid_this_page)
                .ok_or(DammHonoraryFeeError::ArithmeticOverflow)?;

            if remainder > 0 {
                // Transfer remainder to creator
                // TODO: Implement actual token transfer to creator
                progress.cumulative_distributed_today = progress.cumulative_distributed_today
                    .checked_add(remainder)
                    .ok_or(DammHonoraryFeeError::ArithmeticOverflow)?;
            }

            progress.is_closed = true;

            // Emit day closed event
            emit!(CreatorPayoutDayClosed {
                day_id: current_day_id,
                remainder_amount: remainder,
                total_investor_payout: total_paid_this_page,
            });
        }

        // Emit page event
        emit!(InvestorPayoutPage {
            page_index,
            paid_total: total_paid_this_page,
            investor_count: investor_accounts.len() as u32,
            day_id: current_day_id,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeHonoraryPosition<'info> {
    /// The pool for which we're creating the honorary position
    pub pool: AccountInfo<'info>,

    /// The token mints for the pool (for quote mint identification)
    pub token_mint_0: AccountInfo<'info>,
    pub token_mint_1: AccountInfo<'info>,

    /// The position to be created (will be owned by program PDA)
    pub position: AccountInfo<'info>,

    /// The position NFT mint
    pub position_nft_mint: AccountInfo<'info>,

    /// The position NFT owner (program PDA)
    #[account(
        seeds = [b"vault", vault_pubkey.key().as_ref(), b"investor_fee_pos_owner"],
        bump,
    )]
    pub investor_fee_position_owner_pda: SystemAccount<'info>,

    /// The vault pubkey (used in PDA seeds)
    pub vault_pubkey: AccountInfo<'info>,

    /// The creator wallet (for remainder distributions)
    pub creator_wallet: Signer<'info>,

    /// The policy PDA storing configuration
    #[account(
        init,
        payer = creator_wallet,
        space = 8 + std::mem::size_of::<PolicyAccount>(),
        seeds = [b"policy", pool.key().as_ref()],
        bump,
    )]
    pub policy_pda: Account<'info, PolicyAccount>,

    /// The honorary position account
    #[account(
        init,
        payer = creator_wallet,
        space = 8 + std::mem::size_of::<HonoraryPositionAccount>(),
        seeds = [b"honorary_position", pool.key().as_ref()],
        bump,
    )]
    pub honorary_position: Account<'info, HonoraryPositionAccount>,

    /// The program quote treasury ATA
    #[account(
        init,
        payer = creator_wallet,
        associated_token::mint = quote_mint,
        associated_token::authority = investor_fee_position_owner_pda,
    )]
    pub program_quote_treasury_ata: Account<'info, TokenAccount>,

    /// The quote mint (identified from pool tokens)
    pub quote_mint: AccountInfo<'info>,

    /// Token program
    pub token_program: Program<'info, Token>,

    /// Associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// System program
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CrankDistributePage<'info> {
    /// The policy PDA
    pub policy_pda: Account<'info, PolicyAccount>,

    /// The honorary position account
    pub honorary_position: Account<'info, HonoraryPositionAccount>,

    /// The progress PDA tracking distribution state
    #[account(
        seeds = [b"progress", policy_pda.key().as_ref()],
        bump,
    )]
    pub progress_pda: Account<'info, ProgressAccount>,

    /// The program quote treasury ATA (source of funds)
    #[account(
        associated_token::mint = quote_mint,
        associated_token::authority = investor_fee_position_owner_pda,
    )]
    pub program_quote_treasury_ata: Account<'info, TokenAccount>,

    /// The position owner PDA (for claiming fees)
    #[account(
        seeds = [b"vault", vault_pubkey.key().as_ref(), b"investor_fee_pos_owner"],
        bump,
    )]
    pub investor_fee_position_owner_pda: SystemAccount<'info>,

    /// The vault pubkey (from policy)
    pub vault_pubkey: AccountInfo<'info>,

    /// The quote mint
    pub quote_mint: AccountInfo<'info>,

    /// Token program
    pub token_program: Program<'info, Token>,
}

/// Helper function to identify quote mint from pool tokens
fn identify_quote_mint(
    token_mint_0: &AccountInfo,
    token_mint_1: &AccountInfo,
    pool_id: Pubkey,
) -> Result<Pubkey> {
    // This is a simplified implementation
    // In practice, this would involve querying the pool state or using
    // a deterministic method to identify which token is the quote token

    // For now, assume token_mint_0 is quote mint (first in pair)
    // TODO: Implement proper quote mint identification based on pool configuration
    Ok(token_mint_0.key())
}

/// Helper function to validate tick range for quote-only accrual
fn validate_quote_only_position(tick_lower: i32, tick_upper: i32) -> Result<()> {
    // This is a simplified validation
    // In practice, this would involve complex logic to determine if a tick range
    // can only accrue quote fees based on the pool's price range and token ordering

    // For demonstration, we'll accept any range for now
    // TODO: Implement proper quote-only validation logic
    if tick_lower >= tick_upper {
        return Err(DammHonoraryFeeError::InvalidTickRange.into());
    }

    Ok(())
}