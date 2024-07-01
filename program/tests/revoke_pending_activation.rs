#![cfg(feature = "test-sbf")]

mod setup;

use {
    setup::{setup, setup_active_feature, setup_pending_feature},
    solana_feature_gate_program::{
        error::FeatureGateError, instruction::revoke_pending_activation,
    },
    solana_program::instruction::InstructionError,
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        feature::Feature,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::{Transaction, TransactionError},
    },
};

#[tokio::test]
async fn fail_feature_not_signer() {
    let mut context = setup().start_with_context().await;

    let mut instruction = revoke_pending_activation(&Pubkey::new_unique());
    instruction.accounts[0].is_signer = false;

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
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
}

#[tokio::test]
async fn fail_feature_incorrect_owner() {
    let feature_keypair = Keypair::new();

    let mut context = setup().start_with_context().await;

    // Set up a feature account with incorrect owner.
    {
        context.set_account(
            &feature_keypair.pubkey(),
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![0; Feature::size_of()],
                owner: Pubkey::new_unique(), // Incorrect owner.
                ..Account::default()
            }),
        );
    }

    let transaction = Transaction::new_signed_with_payer(
        &[revoke_pending_activation(&feature_keypair.pubkey())],
        Some(&context.payer.pubkey()),
        &[&context.payer, &feature_keypair],
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
        TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
    );
}

#[tokio::test]
async fn fail_feature_invalid_data() {
    let feature_keypair = Keypair::new();

    let mut context = setup().start_with_context().await;

    // Set up a feature account with invalid data.
    {
        context.set_account(
            &feature_keypair.pubkey(),
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![8; Feature::size_of()],
                owner: solana_feature_gate_program::id(),
                ..Account::default()
            }),
        );
    }

    let transaction = Transaction::new_signed_with_payer(
        &[revoke_pending_activation(&feature_keypair.pubkey())],
        Some(&context.payer.pubkey()),
        &[&context.payer, &feature_keypair],
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
        TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
    );
}

#[tokio::test]
async fn fail_feature_already_activated() {
    let feature_keypair = Keypair::new();

    let mut context = setup().start_with_context().await;

    // Set up an active feature account.
    setup_active_feature(&mut context, &feature_keypair.pubkey());

    let transaction = Transaction::new_signed_with_payer(
        &[revoke_pending_activation(&feature_keypair.pubkey())],
        Some(&context.payer.pubkey()),
        &[&context.payer, &feature_keypair],
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
}

#[tokio::test]
async fn success() {
    let feature_keypair = Keypair::new();

    let mut context = setup().start_with_context().await;

    // Set up a pending feature account.
    setup_pending_feature(&mut context, &feature_keypair.pubkey());

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
