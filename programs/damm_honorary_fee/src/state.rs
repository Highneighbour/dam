use anchor_lang::prelude::*;

#[account]
pub struct HonoraryPositionAccount {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner_pda: Pubkey,
    pub quote_mint: Pubkey,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub bump: u8,
}

impl HonoraryPositionAccount {
    pub const SIZE: usize = 32 + 32 + 32 + 32 + 4 + 4 + 1;
}

#[account]
pub struct Policy {
    pub vault: Pubkey,
    pub investor_fee_share_bps: u16,
    pub daily_cap_lamports: Option<u64>,
    pub min_payout_lamports: u64,
    pub allow_create_ata: bool,
    pub y0: u64,
    pub bump: u8,
}

impl Policy {
    pub const SIZE: usize = 32 + 2 + 1 + 8 + 1 + 8 + 1; // Option<u64> stores 1 + 8
}

#[account]
pub struct Progress {
    pub vault: Pubkey,
    pub day_id: u64,
    pub last_distribution_ts: i64,
    pub cumulative_distributed_today: u64,
    pub carry_over_lamports: u64,
    pub cursor_idx: u32,
    pub is_closed: bool,
    pub processed_pages_bitmap: Vec<u8>,
    pub bump: u8,
}

impl Progress {
    pub const MAX_PAGES: usize = 512;
    pub const BITMAP_BYTES: usize = Self::MAX_PAGES / 8; // 64 bytes
    pub const SIZE: usize = 32 + 8 + 8 + 8 + 8 + 4 + 1 + 4 + Self::BITMAP_BYTES + 1; // Vec prefix 4 bytes + data

    pub fn reset_for_new_day(&mut self, day_id: u64, now_ts: i64) {
        self.day_id = day_id;
        self.last_distribution_ts = now_ts;
        self.cumulative_distributed_today = 0;
        self.carry_over_lamports = 0;
        self.cursor_idx = 0;
        self.is_closed = false;
        self.processed_pages_bitmap.fill(0);
    }

    pub fn is_page_processed(&self, idx: u32) -> bool {
        let idx = idx as usize;
        if idx >= Self::MAX_PAGES { return false; }
        let byte = idx / 8;
        let bit = idx % 8;
        (self.processed_pages_bitmap[byte] & (1 << bit)) != 0
    }

    pub fn mark_page_processed(&mut self, idx: u32) {
        let idx = idx as usize;
        let byte = idx / 8;
        let bit = idx % 8;
        self.processed_pages_bitmap[byte] |= 1 << bit;
    }
}

// Marker structs removed; using UncheckedAccount PDAs
