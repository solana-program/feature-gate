#![cfg(feature = "test-sbf")]

mod setup;

use {
    setup::{setup, setup_active_feature, setup_pending_feature, setup_staged_features_account},
    solana_feature_gate_program::{
        error::FeatureGateError,
        instruction::stage_feature_for_activation,
        state::{get_staged_features_address, FeatureStake, StagedFeatures},
    },
    solana_program::instruction::InstructionError,
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        clock::Clock,
        feature::Feature,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        signer::SeedDerivable,
        system_program,
        transaction::{Transaction, TransactionError},
    },
    test_case::test_case,
};

#[tokio::test]
async fn fail_incorrect_authority() {
    let feature = Pubkey::new_unique();
    let authority = Keypair::new(); // Incorrect authority.

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch.saturating_add(1));

    let transaction = Transaction::new_signed_with_payer(
        &[stage_feature_for_activation(
            &feature,
            &staged_features_address,
            &authority.pubkey(),
            true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
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
        TransactionError::InstructionError(0, InstructionError::IncorrectAuthority)
    );
}

#[tokio::test]
async fn fail_authority_not_signer() {
    let feature = Pubkey::new_unique();
    let authority = Keypair::from_seed(&[0u8; 32]).unwrap();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch.saturating_add(1));

    let mut instruction = stage_feature_for_activation(
        &feature,
        &staged_features_address,
        &authority.pubkey(),
        true,
    );
    instruction.accounts[2].is_signer = false; // Authority not signer.

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
    let feature = Pubkey::new_unique();
    let authority = Keypair::from_seed(&[0u8; 32]).unwrap();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch.saturating_add(1));

    // Set up a feature account with incorrect owner.
    {
        context.set_account(
            &feature,
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![0; Feature::size_of()],
                owner: Pubkey::new_unique(), // Incorrect owner.
                ..Account::default()
            }),
        );
    }

    let transaction = Transaction::new_signed_with_payer(
        &[stage_feature_for_activation(
            &feature,
            &staged_features_address,
            &authority.pubkey(),
            true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
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
    let feature = Pubkey::new_unique();
    let authority = Keypair::from_seed(&[0u8; 32]).unwrap();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch.saturating_add(1));

    // Set up a feature account with invalid data.
    {
        context.set_account(
            &feature,
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![8; Feature::size_of()],
                owner: solana_feature_gate_program::id(),
                ..Account::default()
            }),
        );
    }

    let transaction = Transaction::new_signed_with_payer(
        &[stage_feature_for_activation(
            &feature,
            &staged_features_address,
            &authority.pubkey(),
            true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
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
    let feature = Pubkey::new_unique();
    let authority = Keypair::from_seed(&[0u8; 32]).unwrap();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch.saturating_add(1));

    // Set up an active feature account.
    setup_active_feature(&mut context, &feature);

    let transaction = Transaction::new_signed_with_payer(
        &[stage_feature_for_activation(
            &feature,
            &staged_features_address,
            &authority.pubkey(),
            true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
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

#[test_case(true; "initialized")]
#[test_case(false; "not initialized")]
#[tokio::test]
async fn fail_staged_features_incorrect_address(initialized: bool) {
    let feature = Pubkey::new_unique();
    let authority = Keypair::from_seed(&[0u8; 32]).unwrap();

    let mut context = setup().start_with_context().await;

    let staged_features_address = Pubkey::new_unique(); // Incorrect address.

    setup_pending_feature(&mut context, &feature);
    if initialized {
        setup_staged_features_account(&mut context, &staged_features_address, &[]);
    }

    let transaction = Transaction::new_signed_with_payer(
        &[stage_feature_for_activation(
            &feature,
            &staged_features_address,
            &authority.pubkey(),
            !initialized,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
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
            InstructionError::Custom(FeatureGateError::IncorrectStagedFeaturesAddress as u32)
        )
    );
}

#[test_case(true; "initialized")]
#[test_case(false; "not initialized")]
#[tokio::test]
async fn fail_staged_features_incorrect_epoch(initialized: bool) {
    let feature = Pubkey::new_unique();
    let authority = Keypair::from_seed(&[0u8; 32]).unwrap();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let upcoming_epoch = clock.epoch.saturating_add(1);

    let staged_features_address = get_staged_features_address(&upcoming_epoch);

    // Warp to the next epoch, making it the wrong staged features account.
    context.warp_to_epoch(upcoming_epoch).unwrap();

    setup_pending_feature(&mut context, &feature);
    if initialized {
        setup_staged_features_account(&mut context, &staged_features_address, &[]);
    }

    let transaction = Transaction::new_signed_with_payer(
        &[stage_feature_for_activation(
            &feature,
            &staged_features_address,
            &authority.pubkey(),
            !initialized,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
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
            InstructionError::Custom(FeatureGateError::IncorrectStagedFeaturesAddress as u32)
        )
    );
}

#[tokio::test]
async fn fail_staged_features_invalid_data() {
    let feature = Pubkey::new_unique();
    let authority = Keypair::from_seed(&[0u8; 32]).unwrap();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch.saturating_add(1));

    setup_pending_feature(&mut context, &feature);

    // Set up a staged features account with invalid data.
    {
        context.set_account(
            &staged_features_address,
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![8; std::mem::size_of::<StagedFeatures>().saturating_add(1)],
                owner: solana_feature_gate_program::id(),
                ..Account::default()
            }),
        );
    }

    let transaction = Transaction::new_signed_with_payer(
        &[stage_feature_for_activation(
            &feature,
            &staged_features_address,
            &authority.pubkey(),
            false,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
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
async fn fail_feature_already_staged() {
    let feature = Pubkey::new_unique();
    let authority = Keypair::from_seed(&[0u8; 32]).unwrap();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch.saturating_add(1));

    setup_pending_feature(&mut context, &feature);
    setup_staged_features_account(
        &mut context,
        &staged_features_address,
        &[feature], // Feature already staged.
    );

    let transaction = Transaction::new_signed_with_payer(
        &[stage_feature_for_activation(
            &feature,
            &staged_features_address,
            &authority.pubkey(),
            false,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
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
            InstructionError::Custom(FeatureGateError::FeatureAlreadyStaged as u32)
        )
    );
}

#[tokio::test]
async fn fail_feature_stage_full() {
    let feature = Pubkey::new_unique();
    let authority = Keypair::from_seed(&[0u8; 32]).unwrap();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch.saturating_add(1));

    setup_pending_feature(&mut context, &feature);
    setup_staged_features_account(
        &mut context,
        &staged_features_address,
        &[
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(), // Full stage.
        ],
    );

    let transaction = Transaction::new_signed_with_payer(
        &[stage_feature_for_activation(
            &feature,
            &staged_features_address,
            &authority.pubkey(),
            false,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
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
            InstructionError::Custom(FeatureGateError::FeatureStageFull as u32)
        )
    );
}

#[test_case(true; "initialized")]
#[test_case(false; "not initialized")]
#[tokio::test]
async fn success(initialized: bool) {
    let feature = Pubkey::new_unique();
    let authority = Keypair::from_seed(&[0u8; 32]).unwrap();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch.saturating_add(1));

    setup_pending_feature(&mut context, &feature);
    if initialized {
        setup_staged_features_account(&mut context, &staged_features_address, &[]);
    } else {
        // Fund the account with enough rent-exempt lamports.
        let rent = context.banks_client.get_rent().await.unwrap();
        let space = std::mem::size_of::<StagedFeatures>();
        let lamports = rent.minimum_balance(space);
        context.set_account(
            &staged_features_address,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let transaction = Transaction::new_signed_with_payer(
        &[stage_feature_for_activation(
            &feature,
            &staged_features_address,
            &authority.pubkey(),
            !initialized,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert the feature was staged.
    let staged_features_account = context
        .banks_client
        .get_account(staged_features_address)
        .await
        .unwrap()
        .unwrap();
    let staged_features = bytemuck::from_bytes::<StagedFeatures>(&staged_features_account.data);
    assert!(staged_features.features.contains(&FeatureStake {
        feature_id: feature,
        stake_support: 0
    }));
}
