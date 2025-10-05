use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo};

pub mod state;
use state::*;

declare_id!("4L3s2v8u8k4iVwV3s6Zx8u9Q2pY3aG9fQ7wE6bN5mCk7");

#[program]
pub mod cp_amm_stub {
    use super::*;

    pub fn init_pool(ctx: Context<InitPool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.token_base_mint = ctx.accounts.base_mint.key();
        pool.token_quote_mint = ctx.accounts.quote_mint.key();
        pool.bump = ctx.bumps.pool;
        Ok(())
    }

    pub fn create_position(
        ctx: Context<CreatePosition>,
        tick_lower: i32,
        tick_upper: i32,
        quote_only: bool,
    ) -> Result<()> {
        let pos = &mut ctx.accounts.position;
        pos.owner = ctx.accounts.owner.key();
        pos.pool = ctx.accounts.pool.key();
        pos.tick_lower = tick_lower;
        pos.tick_upper = tick_upper;
        pos.quote_only = quote_only;
        pos.accrued_base = 0;
        pos.accrued_quote = 0;
        pos.bump = ctx.bumps.position;
        Ok(())
    }

    pub fn accrue_fees(ctx: Context<AccrueFees>, add_base: u64, add_quote: u64) -> Result<()> {
        let pos = &mut ctx.accounts.position;
        require_keys_eq!(pos.pool, ctx.accounts.pool.key(), AmmError::InvalidPool);
        pos.accrued_base = pos.accrued_base.saturating_add(add_base);
        pos.accrued_quote = pos.accrued_quote.saturating_add(add_quote);
        Ok(())
    }

    // Simulated claim_fees: transfers quote/base to provided treasuries and zeroes accrued
    pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
        let pos = &mut ctx.accounts.position;
        require_keys_eq!(pos.pool, ctx.accounts.pool.key(), AmmError::InvalidPool);

        let base_amount = pos.accrued_base;
        let quote_amount = pos.accrued_quote;

        if base_amount > 0 {
            // Mint base to base_treasury
            let bump = ctx.bumps.pool_signer;
            let pool_key = ctx.accounts.pool.key();
            let seeds: &[&[u8]] = &[b"pool_signer", pool_key.as_ref(), &[bump]];
            let signer = &[seeds];
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.base_mint.to_account_info(),
                    to: ctx.accounts.base_treasury.to_account_info(),
                    authority: ctx.accounts.pool_signer.to_account_info(),
                },
                signer,
            );
            token::mint_to(cpi_ctx, base_amount)?;
        }

        if quote_amount > 0 {
            // Mint quote to quote_treasury
            let bump = ctx.bumps.pool_signer;
            let pool_key = ctx.accounts.pool.key();
            let seeds: &[&[u8]] = &[b"pool_signer", pool_key.as_ref(), &[bump]];
            let signer = &[seeds];
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.quote_mint.to_account_info(),
                    to: ctx.accounts.quote_treasury.to_account_info(),
                    authority: ctx.accounts.pool_signer.to_account_info(),
                },
                signer,
            );
            token::mint_to(cpi_ctx, quote_amount)?;
        }

        pos.accrued_base = 0;
        pos.accrued_quote = 0;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitPool<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Pool::SIZE,
        seeds = [b"pool", base_mint.key().as_ref(), quote_mint.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    pub base_mint: Account<'info, Mint>,
    pub quote_mint: Account<'info, Mint>,
    /// CHECK: signer PDA for mint authority (PDA of this program)
    #[account(seeds = [b"pool_signer", pool.key().as_ref()], bump)]
    pub pool_signer: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatePosition<'info> {
    pub owner: UncheckedAccount<'info>,
    pub pool: Account<'info, Pool>,
    #[account(
        init,
        payer = payer,
        space = 8 + Position::SIZE,
        seeds = [b"position", pool.key().as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AccrueFees<'info> {
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub position: Account<'info, Position>,
}

#[derive(Accounts)]
pub struct ClaimFees<'info> {
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub base_mint: Account<'info, Mint>,
    #[account(mut)]
    pub quote_mint: Account<'info, Mint>,
    #[account(mut)]
    pub base_treasury: Account<'info, TokenAccount>,
    #[account(mut)]
    pub quote_treasury: Account<'info, TokenAccount>,
    /// CHECK: Mint authority signer PDA derived from pool
    #[account(seeds = [b"pool_signer", pool.key().as_ref()], bump)]
    pub pool_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum AmmError {
    #[msg("Invalid pool for position")] InvalidPool,
}
