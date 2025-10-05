use anchor_lang::prelude::*;

pub mod state;
use state::*;

declare_id!("8Wx6m7J4R7dA9cC3G8h5o1bGQJV9oG8X5q7YwF2d3yY3");

#[program]
pub mod streamflow_mock {
    use super::*;

    pub fn set_locked_amount(ctx: Context<SetLockedAmount>, locked_amount: u64, y0: u64) -> Result<()> {
        let lock = &mut ctx.accounts.stream_lock;
        lock.stream_pubkey = ctx.accounts.stream.key();
        lock.locked_amount = locked_amount;
        lock.y0 = y0;
        lock.bump = ctx.bumps.stream_lock;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetLockedAmount<'info> {
    /// CHECK: Arbitrary stream key used as seed; no data read
    pub stream: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + StreamLock::SIZE,
        seeds = [b"stream_lock", stream.key().as_ref()],
        bump,
    )]
    pub stream_lock: Account<'info, StreamLock>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
