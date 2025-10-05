use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct StreamLock {
    pub stream_pubkey: Pubkey,
    pub locked_amount: u64,
    pub y0: u64,
    pub bump: u8,
}

impl StreamLock {
    pub const SIZE: usize = 32 + 8 + 8 + 1;
}
