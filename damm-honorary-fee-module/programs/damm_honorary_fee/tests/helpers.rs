//! Test helpers for DAMM Honorary Fee Module

use solana_program_test::*;
use solana_sdk::{
    signature::Keypair,
    pubkey::Pubkey,
};

/// Create a test context with all necessary accounts
pub async fn create_test_context() -> ProgramTestContext {
    let program_test = ProgramTest::new(
        "damm_honorary_fee",
        damm_honorary_fee::ID,
        None,
    );

    program_test.start().await
}

/// Generate a deterministic pubkey for testing
pub fn test_pubkey(seed: &str) -> Pubkey {
    use solana_sdk::hash::{Hash, Hasher};
    let mut hasher = Hasher::default();
    hasher.hash(seed.as_bytes());
    Pubkey::new_from_array(hasher.result().to_bytes())
}

/// Create test token mints
pub async fn create_test_mints(context: &mut ProgramTestContext) -> (Keypair, Keypair) {
    let quote_mint = Keypair::new();
    let base_mint = Keypair::new();

    // TODO: Actually create the mints using SPL token program

    (quote_mint, base_mint)
}