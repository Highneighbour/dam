use anchor_lang::prelude::*;

#[error_code]
pub enum DammError {
    #[msg("Config would accrue base fees; not quote-only")] NotQuoteOnly,
    #[msg("Base fees observed during claim")] BaseFeesObserved,
    #[msg("Day gate not open for new day")] DayGateNotOpen,
    #[msg("Insufficient treasury balance")] InsufficientTreasury,
    #[msg("Invalid pagination cursor")] InvalidPaginationCursor,
    #[msg("Minimum payout not met; carried forward")] MinPayoutNotMet,
    #[msg("Streamflow read error")] StreamflowReadError,
    #[msg("Already processed page")] AlreadyProcessedPage,
    #[msg("Invalid quote mint for pool")] InvalidQuoteMint,
}
