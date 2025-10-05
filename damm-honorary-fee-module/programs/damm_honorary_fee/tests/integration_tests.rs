//! Integration tests for DAMM Honorary Fee Module
//!
//! These tests run against a local solana-test-validator and exercise
//! the complete flow from initialization to fee distribution.

use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signer},
    transaction::Transaction,
    pubkey::Pubkey,
};

use damm_honorary_fee::{
    state::*,
    events::*,
};
use damm_honorary_fee::damm_honorary_fee::*;

mod helpers;

#[tokio::test]
async fn test_initialize_honorary_position() {
    let program_id = damm_honorary_fee::ID;
    let mut context = setup_test_context().await;

    // Generate test accounts
    let pool_id = Pubkey::new_unique();
    let vault_pubkey = Pubkey::new_unique();
    let creator_wallet = Keypair::new();

    // Create token mints (mock)
    let quote_mint = Keypair::new();
    let base_mint = Keypair::new();

    // Create test context with accounts
    let mut accounts = Vec::new();

    // Add program accounts for initialization
    let policy_pda = Pubkey::find_program_address(
        &[b"policy", pool_id.as_ref()],
        &program_id,
    ).0;

    let honorary_position_pda = Pubkey::find_program_address(
        &[b"honorary_position", pool_id.as_ref()],
        &program_id,
    ).0;

    let investor_fee_position_owner_pda = Pubkey::find_program_address(
        &[b"vault", vault_pubkey.as_ref(), b"investor_fee_pos_owner"],
        &program_id,
    ).0;

    // Create the initialize instruction
    let initialize_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(pool_id, false),
            AccountMeta::new_readonly(quote_mint.pubkey(), false),
            AccountMeta::new_readonly(base_mint.pubkey(), false),
            AccountMeta::new_readonly(honorary_position_pda, false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false), // position nft mint
            AccountMeta::new(investor_fee_position_owner_pda, false),
            AccountMeta::new_readonly(vault_pubkey, false),
            AccountMeta::new_readonly(creator_wallet.pubkey(), true),
            AccountMeta::new(policy_pda, false),
            AccountMeta::new(honorary_position_pda, false),
            AccountMeta::new(Pubkey::new_unique(), false), // program quote treasury ata
            AccountMeta::new_readonly(quote_mint.pubkey(), false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: [
            0, // discriminator for initialize_honorary_position
            // TODO: Properly serialize the instruction data
        ].to_vec(),
    };

    // Process the transaction
    let tx = Transaction::new_signed(
        &creator_wallet,
        &[initialize_ix],
        context.last_blockhash,
        context.payer.pubkey(),
    );

    // This should succeed
    assert!(context.banks_client.process_transaction(tx).await.is_ok());

    // Verify the accounts were created correctly
    let policy_account = context.banks_client.get_account(policy_pda).await.unwrap().unwrap();
    let policy_data: PolicyAccount = PolicyAccount::try_deserialize(&mut policy_account.data.as_ref()).unwrap();

    assert_eq!(policy_data.pool_id, pool_id);
    assert_eq!(policy_data.vault_pubkey, vault_pubkey);
    assert_eq!(policy_data.creator_wallet, creator_wallet.pubkey());
    assert_eq!(policy_data.quote_mint, quote_mint.pubkey());
    assert_eq!(policy_data.investor_fee_share_bps, 5000);
    assert_eq!(policy_data.daily_cap_lamports, Some(1_000_000_000));
    assert_eq!(policy_data.min_payout_lamports, 100_000);
    assert_eq!(policy_data.y0_total_allocation, 1_000_000_000);
}

#[tokio::test]
async fn test_crank_distribute_page() {
    // TODO: Implement crank distribution test
    // This would involve:
    // 1. Setting up a pool with fees
    // 2. Creating an honorary position
    // 3. Setting locked amounts in the mock Streamflow program
    // 4. Calling the crank function
    // 5. Verifying distributions
}

#[tokio::test]
async fn test_quote_only_validation() {
    // TODO: Test that invalid tick ranges are rejected
}

#[tokio::test]
async fn test_pagination_idempotency() {
    // TODO: Test that re-running the same page doesn't double-pay
}

async fn setup_test_context() -> ProgramTestContext {
    let mut program_test = ProgramTest::new(
        "damm_honorary_fee",
        damm_honorary_fee::ID,
        None,
    );

    program_test.start().await
}