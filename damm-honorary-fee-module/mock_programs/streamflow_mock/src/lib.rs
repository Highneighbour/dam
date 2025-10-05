//! Mock Streamflow program for testing DAMM fee distribution
//!
//! This program provides a simple interface to set and query locked amounts
//! for stream pubkeys, simulating the Streamflow program's locked amount queries.

use anchor_lang::prelude::*;

declare_id!("StreamMock11111111111111111111111111111111");

#[program]
pub mod streamflow_mock {
    use super::*;

    /// Set the locked amount for a stream pubkey (for testing)
    pub fn set_locked_amount(
        ctx: Context<SetLockedAmount>,
        stream_pubkey: Pubkey,
        locked_amount: u64,
    ) -> Result<()> {
        let locked_account = &mut ctx.accounts.locked_account;
        locked_account.stream_pubkey = stream_pubkey;
        locked_account.locked_amount = locked_amount;
        Ok(())
    }

    /// Get the locked amount for a stream pubkey
    pub fn get_locked_amount(
        ctx: Context<GetLockedAmount>,
        stream_pubkey: Pubkey,
    ) -> Result<u64> {
        let locked_account = &ctx.accounts.locked_account;
        if locked_account.stream_pubkey != stream_pubkey {
            return Err(StreamflowMockError::StreamNotFound.into());
        }
        Ok(locked_account.locked_amount)
    }
}

#[derive(Accounts)]
#[instruction(stream_pubkey: Pubkey)]
pub struct SetLockedAmount<'info> {
    /// The locked amount storage account
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + std::mem::size_of::<LockedAmountAccount>(),
        seeds = [b"locked", stream_pubkey.as_ref()],
        bump,
    )]
    pub locked_account: Account<'info, LockedAmountAccount>,

    /// Authority to set locked amounts
    pub authority: Signer<'info>,

    /// System program
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(stream_pubkey: Pubkey)]
pub struct GetLockedAmount<'info> {
    /// The locked amount storage account
    #[account(
        seeds = [b"locked", stream_pubkey.as_ref()],
        bump,
    )]
    pub locked_account: Account<'info, LockedAmountAccount>,
}

/// Storage account for locked amounts per stream
#[account]
pub struct LockedAmountAccount {
    /// The stream pubkey this locked amount applies to
    pub stream_pubkey: Pubkey,
    /// The locked amount
    pub locked_amount: u64,
}

#[error_code]
pub enum StreamflowMockError {
    #[msg("Stream not found")]
    StreamNotFound,
}