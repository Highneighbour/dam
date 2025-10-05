use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

pub mod errors;
pub mod events;
pub mod state;

use errors::*;
use events::*;
use state::*;

// NOTE: This is a placeholder; replaced by Anchor.toml during localnet deploy
declare_id!("DHCtQhk3J4Y3GQxYbJAm1H1eVDqHvGkV8PH4YyC4NujG");

#[program]
pub mod damm_honorary_fee {
    use super::*;

    pub fn initialize_honorary_position(
        ctx: Context<InitializeHonoraryPosition>,
        _pool_id: Pubkey,
        tick_lower: i32,
        tick_upper: i32,
        vault_pubkey: Pubkey,
        investor_fee_share_bps: u16,
        daily_cap_lamports: Option<u64>,
        min_payout_lamports: u64,
        y0: u64,
    ) -> Result<()> {
        // Determine quote mint from cp-amm pool account
        let pool = &ctx.accounts.cp_amm_pool;
        let quote_mint = pool.token_quote_mint;
        require_keys_eq!(quote_mint, ctx.accounts.quote_mint.key(), DammError::InvalidQuoteMint);

        // Preflight: ensure quote-only authorized by cp-amm metadata (stub semantics)
        let position = &ctx.accounts.cp_amm_position;
        require!(position.quote_only, DammError::NotQuoteOnly);

        // Write HonoraryPositionAccount
        let pos = &mut ctx.accounts.honorary_position;
        pos.pool = pool.key();
        pos.position = position.key();
        pos.owner_pda = ctx.accounts.investor_fee_pos_owner.key();
        pos.quote_mint = quote_mint;
        pos.tick_lower = tick_lower;
        pos.tick_upper = tick_upper;
        pos.bump = ctx.bumps.honorary_position;

        // Policy
        let policy = &mut ctx.accounts.policy;
        policy.vault = vault_pubkey;
        policy.investor_fee_share_bps = investor_fee_share_bps;
        policy.daily_cap_lamports = daily_cap_lamports;
        policy.min_payout_lamports = min_payout_lamports;
        policy.allow_create_ata = true; // default allow
        policy.y0 = y0;
        policy.bump = ctx.bumps.policy;

        // Progress
        let progress = &mut ctx.accounts.progress;
        progress.vault = vault_pubkey;
        progress.day_id = 0;
        progress.last_distribution_ts = 0;
        progress.cumulative_distributed_today = 0;
        progress.carry_over_lamports = 0;
        progress.cursor_idx = 0;
        progress.is_closed = true;
        progress.processed_pages_bitmap = vec![0u8; Progress::BITMAP_BYTES];
        progress.bump = ctx.bumps.progress;

        emit!(HonoraryPositionInitialized {
            pool: pos.pool,
            position: pos.position,
            quote_mint,
            tick_lower,
            tick_upper,
        });

        Ok(())
    }

    pub fn crank_distribute_page<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CrankDistributePage<'info>>,
        page_index: u32,
        is_final_page_in_day: bool,
    ) -> Result<()> {
        let now_ts = Clock::get()?.unix_timestamp;
        let day_id = if now_ts >= 0 { (now_ts as u64) / 86_400 } else { 0 };

        // Day gating logic
        let progress = &mut ctx.accounts.progress;
        if progress.day_id != day_id {
            // If last day closed, allow new day only if 24h elapsed since last_distribution_ts
            if progress.last_distribution_ts != 0 && now_ts < progress.last_distribution_ts + 86_400 {
                return err!(DammError::DayGateNotOpen);
            }
            progress.reset_for_new_day(day_id, now_ts);
        } else {
            // Same day: ensure we haven't passed the 24h window for first crank
            if progress.cursor_idx == 0 && now_ts < progress.last_distribution_ts + 86_400 {
                // ok first crank of day; last_distribution_ts was set on reset
            }
        }

        // Idempotency check
        if progress.is_page_processed(page_index) {
            emit!(InvestorPayoutPage { page_index, paid_total: 0 });
            return Ok(());
        }

        if page_index != progress.cursor_idx {
            return err!(DammError::InvalidPaginationCursor);
        }

        // Record treasury balances before claim
        let pre_quote = ctx.accounts.program_quote_treasury.amount;
        let pre_base = ctx.accounts.program_base_treasury.amount;

        // CPI to cp-amm stub to claim fees into treasuries
        {
            let cpi_program = ctx.accounts.cp_amm_program.to_account_info();
            let cpi_accounts = cp_amm_stub::cpi::accounts::ClaimFees {
                pool: ctx.accounts.cp_amm_pool.to_account_info(),
                position: ctx.accounts.cp_amm_position.to_account_info(),
                base_mint: ctx.accounts.base_mint.to_account_info(),
                quote_mint: ctx.accounts.quote_mint.to_account_info(),
                base_treasury: ctx.accounts.program_base_treasury.to_account_info(),
                quote_treasury: ctx.accounts.program_quote_treasury.to_account_info(),
                pool_signer: ctx.accounts.cp_amm_pool_signer.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            cp_amm_stub::cpi::claim_fees(cpi_ctx)?;
        }

        let post_quote = ctx.accounts.program_quote_treasury.amount;
        let post_base = ctx.accounts.program_base_treasury.amount;
        let claimed_quote = post_quote.saturating_sub(pre_quote);
        let claimed_base = post_base.saturating_sub(pre_base);

        if claimed_base > 0 {
            return err!(DammError::BaseFeesObserved);
        }

        emit!(QuoteFeesClaimed { amount: claimed_quote });

        // Read streamflow locked amounts (first pass)
        let mut locked_total: u128 = 0;
        let mut locks: Vec<(usize, u64)> = Vec::new(); // (investor_ata_index, locked_i)
        let rem: &'info [AccountInfo<'info>] = &ctx.remaining_accounts;
        for i in (0..rem.len()).step_by(2) {
            if i + 1 >= rem.len() { break; }
            let stream_ai = &rem[i];
            require_keys_eq!(*stream_ai.owner, ctx.accounts.streamflow_program.key(), DammError::StreamflowReadError);
            let data = stream_ai.data.borrow();
            if data.len() < 8 + 32 + 8 + 8 + 1 { return err!(DammError::StreamflowReadError); }
            let locked = u64::from_le_bytes(data[8+32..8+32+8].try_into().unwrap());
            locks.push((i + 1, locked));
            locked_total = locked_total.saturating_add(locked as u128);
        }

        let policy = &mut ctx.accounts.policy;
        let y0 = policy.y0 as u128;
        let f_locked_bps = if y0 == 0 { 0u128 } else { locked_total.saturating_mul(10_000).checked_div(y0).unwrap_or(0) };
        let eligible_bps = core::cmp::min(policy.investor_fee_share_bps as u128, f_locked_bps);
        let mut investor_fee_quote: u128 = (claimed_quote as u128).saturating_mul(eligible_bps) / 10_000u128;

        // Apply daily cap
        if let Some(cap) = policy.daily_cap_lamports {
            let remaining_cap = cap.saturating_sub(progress.cumulative_distributed_today);
            if investor_fee_quote > remaining_cap as u128 {
                investor_fee_quote = remaining_cap as u128;
            }
        }

        let mut paid_total: u128 = 0;
        let mut carry_over: u128 = progress.carry_over_lamports as u128;

        if locked_total == 0 || investor_fee_quote == 0 {
            // Nothing to pay to investors; carry everything forward (to creator at close)
            carry_over = carry_over.saturating_add(investor_fee_quote);
        } else {
            let (treasury_pda, treasury_bump) = Pubkey::find_program_address(
                &[b"treasury", policy.vault.as_ref()],
                &crate::ID,
            );
            require_keys_eq!(treasury_pda, ctx.accounts.treasury_authority.key(), DammError::InsufficientTreasury);
            for (investor_ata_idx, locked_i) in locks.iter() {
                if *locked_i == 0 { continue; }
                let weight_num = *locked_i as u128;
                let payout_i: u128 = (investor_fee_quote.saturating_mul(weight_num)) / locked_total;
                if payout_i == 0 || payout_i < policy.min_payout_lamports as u128 {
                    carry_over = carry_over.saturating_add(payout_i);
                    continue;
                }
                // Transfer from program_quote_treasury to investor
                let seeds: &[&[u8]] = &[b"treasury", policy.vault.as_ref(), &[treasury_bump]];
                let signer = &[seeds];
                // Expect the investor ATA at index investor_ata_idx
                let investor_ata_ai = &rem[*investor_ata_idx];
                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.program_quote_treasury.to_account_info(),
                        to: investor_ata_ai.clone(),
                        authority: ctx.accounts.treasury_authority.to_account_info(),
                    },
                    signer,
                );
                token::transfer(cpi_ctx, payout_i as u64)?;
                paid_total = paid_total.saturating_add(payout_i);
            }
        }

        progress.carry_over_lamports = carry_over as u64;
        progress.cumulative_distributed_today = progress.cumulative_distributed_today.saturating_add(paid_total as u64);
        progress.mark_page_processed(page_index);
        progress.cursor_idx = progress.cursor_idx.saturating_add(1);
        progress.last_distribution_ts = now_ts;

        emit!(InvestorPayoutPage { page_index, paid_total: paid_total as u64 });

        if is_final_page_in_day {
            // Send remainder to creator
            let remainder = (claimed_quote as u128)
                .saturating_sub(paid_total)
                .saturating_add(progress.carry_over_lamports as u128);
            if remainder > 0 {
                let (_treasury_pda, treasury_bump) = Pubkey::find_program_address(
                    &[b"treasury", policy.vault.as_ref()],
                    &crate::ID,
                );
                let seeds: &[&[u8]] = &[b"treasury", policy.vault.as_ref(), &[treasury_bump]];
                let signer = &[seeds];
                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.program_quote_treasury.to_account_info(),
                        to: ctx.accounts.creator_quote_ata.to_account_info(),
                        authority: ctx.accounts.treasury_authority.to_account_info(),
                    },
                    signer,
                );
                token::transfer(cpi_ctx, remainder as u64)?;
            }
            progress.is_closed = true;
            emit!(CreatorPayoutDayClosed { day_id: progress.day_id, remainder: remainder as u64 });
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeHonoraryPosition<'info> {
    // CP-AMM pool metadata (stub)
    pub cp_amm_pool: Account<'info, cp_amm_stub::state::Pool>,
    pub cp_amm_position: Account<'info, cp_amm_stub::state::Position>,

    // Honorary position state
    #[account(
        init,
        payer = payer,
        space = 8 + HonoraryPositionAccount::SIZE,
        seeds = [b"honorary_position", cp_amm_pool.key().as_ref()],
        bump,
    )]
    pub honorary_position: Account<'info, HonoraryPositionAccount>,

    /// PDA owner of the honorary position
    /// CHECK: PDA, no data account needed
    #[account(seeds = [b"vault", vault.key().as_ref(), b"investor_fee_pos_owner"], bump)]
    pub investor_fee_pos_owner: UncheckedAccount<'info>,

    /// Policy config PDA
    #[account(
        init,
        payer = payer,
        space = 8 + Policy::SIZE,
        seeds = [b"policy", vault.key().as_ref()],
        bump,
    )]
    pub policy: Account<'info, Policy>,

    /// Daily progress PDA
    #[account(
        init,
        payer = payer,
        space = 8 + Progress::SIZE,
        seeds = [b"progress", vault.key().as_ref()],
        bump,
    )]
    pub progress: Account<'info, Progress>,

    /// Treasury authority PDA
    /// CHECK: PDA, no data account needed
    #[account(seeds = [b"treasury", vault.key().as_ref()], bump)]
    pub treasury_authority: UncheckedAccount<'info>,

    pub quote_mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Vault as key-only, externally managed
    pub vault: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CrankDistributePage<'info> {
    #[account(mut)]
    pub policy: Account<'info, Policy>,
    #[account(mut)]
    pub progress: Account<'info, Progress>,

    // Amm
    pub cp_amm_program: Program<'info, cp_amm_stub::program::CpAmmStub>,
    pub cp_amm_pool: Account<'info, cp_amm_stub::state::Pool>,
    #[account(mut)]
    pub cp_amm_position: Account<'info, cp_amm_stub::state::Position>,
    #[account(mut)]
    pub base_mint: Account<'info, Mint>,
    #[account(mut)]
    pub quote_mint: Account<'info, Mint>,
    #[account(mut)]
    pub program_base_treasury: Account<'info, TokenAccount>,
    #[account(mut)]
    pub program_quote_treasury: Account<'info, TokenAccount>,
    /// CHECK: pool signer used by cp-amm
    pub cp_amm_pool_signer: UncheckedAccount<'info>,

    // Treasury authority
    /// CHECK: PDA as authority for treasuries
    pub treasury_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator_quote_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Streamflow program id
    pub streamflow_program: UncheckedAccount<'info>,
}
