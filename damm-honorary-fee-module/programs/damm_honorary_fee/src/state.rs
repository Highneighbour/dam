//! State account definitions for the DAMM honorary fee module

use anchor_lang::prelude::*;

/// Policy configuration for fee distribution
#[account]
pub struct PolicyAccount {
    /// The pool this policy applies to
    pub pool_id: Pubkey,
    /// The vault pubkey (used in PDA seeds)
    pub vault_pubkey: Pubkey,
    /// The creator wallet (receives remainder)
    pub creator_wallet: Pubkey,
    /// The quote mint for this pool
    pub quote_mint: Pubkey,
    /// The investor fee share in basis points (max 10000 = 100%)
    pub investor_fee_share_bps: u16,
    /// Daily cap on total distribution in lamports (optional)
    pub daily_cap_lamports: Option<u64>,
    /// Minimum payout per investor in lamports
    pub min_payout_lamports: u64,
    /// Total investor allocation at TGE (Y0)
    pub y0_total_allocation: u64,
    /// Bump for PDA
    pub bump: u8,
}

/// Honorary position metadata
#[account]
pub struct HonoraryPositionAccount {
    /// The pool this position belongs to
    pub pool_id: Pubkey,
    /// The position pubkey
    pub position_id: Pubkey,
    /// The position NFT mint
    pub position_nft_mint: Pubkey,
    /// The owner PDA that controls this position
    pub owner_pda: Pubkey,
    /// The quote mint for this pool
    pub quote_mint: Pubkey,
    /// The tick range for this position
    pub tick_lower: i32,
    pub tick_upper: i32,
    /// Bump for PDA
    pub bump: u8,
}

/// Progress tracking for daily distribution
#[account]
pub struct ProgressAccount {
    /// The policy this progress tracks
    pub policy_id: Pubkey,
    /// Current day ID (floor(timestamp / 86400))
    pub day_id: u64,
    /// Last distribution timestamp
    pub last_distribution_ts: i64,
    /// Total distributed today in lamports
    pub cumulative_distributed_today: u64,
    /// Carry over lamports for next distribution
    pub carry_over_lamports: u64,
    /// Current page index for pagination
    pub cursor_idx: u32,
    /// Whether this day's distribution is closed
    pub is_closed: bool,
    /// Page payout tracking (page_index -> total_paid_this_page)
    pub page_payouts: std::collections::BTreeMap<u32, u64>,
    /// Bump for PDA
    pub bump: u8,
}

/// Investor data for a distribution page
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InvestorAccount {
    /// The investor's quote ATA
    pub investor_quote_ata: Pubkey,
    /// The Streamflow stream pubkey for this investor
    pub stream_pubkey: Pubkey,
    /// Locked amount for this investor (queried from Streamflow)
    pub locked_amount: u64,
}