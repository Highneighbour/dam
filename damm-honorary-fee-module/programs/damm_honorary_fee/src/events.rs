//! Event definitions for the DAMM honorary fee module

use anchor_lang::prelude::*;

/// Event emitted when an honorary position is initialized
#[event]
pub struct HonoraryPositionInitialized {
    /// The pool ID
    pub pool_id: Pubkey,
    /// The position ID
    pub position_id: Pubkey,
    /// The position NFT mint
    pub position_nft_mint: Pubkey,
    /// The owner PDA
    pub owner_pda: Pubkey,
    /// The quote mint
    pub quote_mint: Pubkey,
    /// The tick range
    pub tick_lower: i32,
    pub tick_upper: i32,
}

/// Event emitted when quote fees are claimed
#[event]
pub struct QuoteFeesClaimed {
    /// The amount of quote fees claimed
    pub amount: u64,
    /// The pool ID
    pub pool_id: Pubkey,
}

/// Event emitted for each page of investor payouts
#[event]
pub struct InvestorPayoutPage {
    /// The page index
    pub page_index: u32,
    /// Total paid to investors in this page
    pub paid_total: u64,
    /// Number of investors in this page
    pub investor_count: u32,
    /// Current day ID
    pub day_id: u64,
}

/// Event emitted when a day's distribution is closed
#[event]
pub struct CreatorPayoutDayClosed {
    /// The day ID
    pub day_id: u64,
    /// The remainder amount sent to creator
    pub remainder_amount: u64,
    /// Total distributed to investors this day
    pub total_investor_payout: u64,
}

/// Event emitted for individual investor payouts
#[event]
pub struct InvestorPayout {
    /// The investor's quote ATA
    pub investor_quote_ata: Pubkey,
    /// The amount paid to this investor
    pub amount: u64,
    /// The locked amount that determined their share
    pub locked_amount: u64,
    /// The page index this payout was part of
    pub page_index: u32,
}