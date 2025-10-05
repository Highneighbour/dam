//! Error types for the DAMM honorary fee module

use anchor_lang::prelude::*;

#[error_code]
pub enum DammHonoraryFeeError {
    #[msg("The position configuration may accrue base fees - not quote-only")]
    NotQuoteOnly,

    #[msg("Base fees were observed during claim - distribution aborted")]
    BaseFeesObserved,

    #[msg("Distribution day gate is not open - too early for new day")]
    DayGateNotOpen,

    #[msg("Insufficient treasury balance for distribution")]
    InsufficientTreasury,

    #[msg("Invalid pagination cursor - page already processed or out of bounds")]
    InvalidPaginationCursor,

    #[msg("Payout below minimum threshold - carried forward")]
    MinPayoutNotMet,

    #[msg("Failed to read from Streamflow program")]
    StreamflowReadError,

    #[msg("Failed to create associated token account")]
    AtaCreationFailed,

    #[msg("Invalid pool token order - cannot determine quote mint")]
    InvalidPoolTokenOrder,

    #[msg("Position tick range validation failed")]
    InvalidTickRange,

    #[msg("Daily cap exceeded")]
    DailyCapExceeded,

    #[msg("Arithmetic overflow in fee calculation")]
    ArithmeticOverflow,

    #[msg("Unauthorized access to program function")]
    Unauthorized,
}