use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub token_base_mint: Pubkey,
    pub token_quote_mint: Pubkey,
    pub bump: u8,
}

impl Pool {
    pub const SIZE: usize = 32 + 32 + 1;
}

#[account]
pub struct Position {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub quote_only: bool,
    pub accrued_base: u64,
    pub accrued_quote: u64,
    pub bump: u8,
}

impl Position {
    pub const SIZE: usize = 32 + 32 + 4 + 4 + 1 + 8 + 8 + 1;
}
