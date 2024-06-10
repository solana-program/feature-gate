#![cfg(feature = "test-sbf")]

use {
    solana_feature_gate_program::{
        error::FeatureGateError, instruction::revoke_pending_activation,
    },
    solana_program::instruction::InstructionError,
    solana_program_test::{processor, tokio, ProgramTest, ProgramTestContext},
    solana_sdk::{
        account::Account as SolanaAccount,
        feature::{activate_with_lamports, Feature},
        signature::{Keypair, Signer},
        transaction::{Transaction, TransactionError},
    },
};

async fn setup_pending_feature(
    context: &mut ProgramTestContext,
    feature_keypair: &Keypair,
    rent_lamports: u64,
) {
    let transaction = Transaction::new_signed_with_payer(
        &activate_with_lamports(
            &feature_keypair.pubkey(),
            &context.payer.pubkey(),
            rent_lamports,
        ),
        Some(&context.payer.pubkey()),
        &[&context.payer, feature_keypair],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_revoke_pending_activation() {
    let feature_keypair = Keypair::new();
    let mock_active_feature_keypair = Keypair::new();

    let mut program_test = ProgramTest::new(
        "solana_feature_gate_program",
        solana_feature_gate_program::id(),
        processor!(solana_feature_gate_program::processor::process),
    );

    // Add a mock _active_ feature for testing later
    program_test.add_account(
        mock_active_feature_keypair.pubkey(),
        SolanaAccount {
            lamports: 500_000_000,
            owner: solana_feature_gate_program::id(),
            data: vec![
                1, // `Some()`
                45, 0, 0, 0, 0, 0, 0, 0, // Random slot `u64`
            ],
            ..SolanaAccount::default()
        },
    );

    let mut context = program_test.start_with_context().await;
    let rent = context.banks_client.get_rent().await.unwrap();
    let rent_lamports = rent.minimum_balance(Feature::size_of()); // For checking account balance later

    setup_pending_feature(&mut context, &feature_keypair, rent_lamports).await;

    // Fail: feature not signer
    let mut revoke_ix = revoke_pending_activation(&feature_keypair.pubkey());
    revoke_ix.accounts[0].is_signer = false;
    let transaction = Transaction::new_signed_with_payer(
        &[revoke_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let error = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        error,
        TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
    );

    // Fail: feature is already active
    let transaction = Transaction::new_signed_with_payer(
        &[revoke_pending_activation(
            &mock_active_feature_keypair.pubkey(),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mock_active_feature_keypair],
        context.last_blockhash,
    );
    let error = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        error,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(FeatureGateError::FeatureAlreadyActivated as u32)
        )
    );

    // Success: Revoke a feature activation
    let transaction = Transaction::new_signed_with_payer(
        &[revoke_pending_activation(&feature_keypair.pubkey())],
        Some(&context.payer.pubkey()),
        &[&context.payer, &feature_keypair],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Confirm feature account was closed.
    let feature_account = context
        .banks_client
        .get_account(feature_keypair.pubkey())
        .await
        .unwrap();
    assert!(feature_account.is_none());
}
