use anchor_lang::prelude::*;

#[event]
pub struct HonoraryPositionInitialized {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub quote_mint: Pubkey,
    pub tick_lower: i32,
    pub tick_upper: i32,
}

#[event]
pub struct QuoteFeesClaimed {
    pub amount: u64,
}

#[event]
pub struct InvestorPayoutPage {
    pub page_index: u32,
    pub paid_total: u64,
}

#[event]
pub struct CreatorPayoutDayClosed {
    pub day_id: u64,
    pub remainder: u64,
}
