#![cfg(feature = "test-sbf")]

mod setup;

use {
    setup::{setup, setup_staged_features_account},
    solana_feature_gate_program::{
        error::FeatureGateError,
        instruction::signal_support_for_staged_features,
        state::{get_staged_features_address, FeatureBitMask, StagedFeatures},
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
};

#[tokio::test]
async fn fail_vote_account_not_signer() {
    let vote_account = Keypair::new();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch);

    let mut instruction = signal_support_for_staged_features(
        &staged_features_address,
        &vote_account.pubkey(),
        FeatureBitMask(0),
    );
    instruction.accounts[1].is_signer = false; // Vote account not signer.

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
async fn fail_staged_features_incorrect_address() {
    let vote_account = Keypair::new();

    let mut context = setup().start_with_context().await;

    let staged_features_address = Pubkey::new_unique(); // Incorrect address.

    setup_staged_features_account(&mut context, &staged_features_address, &[]);

    let transaction = Transaction::new_signed_with_payer(
        &[signal_support_for_staged_features(
            &staged_features_address,
            &vote_account.pubkey(),
            FeatureBitMask(0),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &vote_account],
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
    let vote_account = Keypair::new();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch);

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
        &[signal_support_for_staged_features(
            &staged_features_address,
            &vote_account.pubkey(),
            FeatureBitMask(0),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &vote_account],
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
async fn success_not_a_vote_account() {
    let vote_account = Keypair::new();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch);

    setup_staged_features_account(&mut context, &staged_features_address, &[]);

    // Set up the vote account as a system account.
    {
        context.set_account(
            &vote_account.pubkey(),
            &AccountSharedData::new(100_000_000, 0, &system_program::id()),
        );
    }

    // Capture the initial state of the staged features account.
    let staged_features_account = context
        .banks_client
        .get_account(staged_features_address)
        .await
        .unwrap()
        .unwrap();
    let initial_staged_features =
        bytemuck::from_bytes::<StagedFeatures>(&staged_features_account.data);

    let transaction = Transaction::new_signed_with_payer(
        &[signal_support_for_staged_features(
            &staged_features_address,
            &vote_account.pubkey(),
            FeatureBitMask(0),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &vote_account],
        context.last_blockhash,
    );

    // The program should succeed.
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert the staged features account was not modified.
    let staged_features_account = context
        .banks_client
        .get_account(staged_features_address)
        .await
        .unwrap()
        .unwrap();
    let staged_features = bytemuck::from_bytes::<StagedFeatures>(&staged_features_account.data);

    assert_eq!(initial_staged_features, staged_features);
}

#[tokio::test]
async fn success_vote_account_no_stake() {
    let vote_account = Keypair::new();

    let mut context = setup().start_with_context().await;

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    let staged_features_address = get_staged_features_address(&clock.epoch);

    setup_staged_features_account(&mut context, &staged_features_address, &[]);

    // TODO: Change this step to set up an actual vote account with no stake.
    // The rest of the test is the same.
    //
    // Set up the vote account as a system account.
    {
        context.set_account(
            &vote_account.pubkey(),
            &AccountSharedData::new(100_000_000, 0, &system_program::id()),
        );
    }

    // Capture the initial state of the staged features account.
    let staged_features_account = context
        .banks_client
        .get_account(staged_features_address)
        .await
        .unwrap()
        .unwrap();
    let initial_staged_features =
        bytemuck::from_bytes::<StagedFeatures>(&staged_features_account.data);

    let transaction = Transaction::new_signed_with_payer(
        &[signal_support_for_staged_features(
            &staged_features_address,
            &vote_account.pubkey(),
            FeatureBitMask(0),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &vote_account],
        context.last_blockhash,
    );

    // The program should succeed.
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert the staged features account was not modified.
    let staged_features_account = context
        .banks_client
        .get_account(staged_features_address)
        .await
        .unwrap()
        .unwrap();
    let staged_features = bytemuck::from_bytes::<StagedFeatures>(&staged_features_account.data);

    assert_eq!(initial_staged_features, staged_features);
}

#[tokio::test]
async fn success() {
    let feature_keypairs = [
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
    ];
    let authority = Keypair::from_seed(&[0u8; 32]).unwrap();
    let _vote_account = Keypair::new();

    let mut context = setup().start_with_context().await;

    let next_epoch = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .epoch
        .saturating_add(1);

    let staged_features_address = get_staged_features_address(&next_epoch);

    let rent = context.banks_client.get_rent().await.unwrap();

    // We can't warp and also use `context.set_account`, so do the setup with
    // transactions instead.
    //
    // Create the first four features.
    {
        let instructions = feature_keypairs[..4]
            .iter()
            .flat_map(|feature_keypair| {
                solana_sdk::feature::activate_with_lamports(
                    &feature_keypair.pubkey(),
                    &context.payer.pubkey(),
                    rent.minimum_balance(Feature::size_of()),
                )
            })
            .collect::<Vec<_>>();

        context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &instructions,
                Some(&context.payer.pubkey()),
                &[
                    &context.payer,
                    &feature_keypairs[0],
                    &feature_keypairs[1],
                    &feature_keypairs[2],
                    &feature_keypairs[3],
                ],
                context.last_blockhash,
            ))
            .await
            .unwrap();
    }
    // Set up the remaining four features.
    {
        let instructions = feature_keypairs[4..]
            .iter()
            .flat_map(|feature_keypair| {
                solana_sdk::feature::activate_with_lamports(
                    &feature_keypair.pubkey(),
                    &context.payer.pubkey(),
                    rent.minimum_balance(Feature::size_of()),
                )
            })
            .collect::<Vec<_>>();

        context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &instructions,
                Some(&context.payer.pubkey()),
                &[
                    &context.payer,
                    &feature_keypairs[4],
                    &feature_keypairs[5],
                    &feature_keypairs[6],
                    &feature_keypairs[7],
                ],
                context.last_blockhash,
            ))
            .await
            .unwrap();
    }
    // Set up the staged features account.
    {
        let mut instructions = vec![solana_sdk::system_instruction::transfer(
            &context.payer.pubkey(),
            &staged_features_address,
            rent.minimum_balance(std::mem::size_of::<StagedFeatures>()),
        )];

        for feature_keypair in &feature_keypairs {
            instructions.push(
                solana_feature_gate_program::instruction::stage_feature_for_activation(
                    &feature_keypair.pubkey(),
                    &staged_features_address,
                    &authority.pubkey(),
                    true,
                ),
            );
        }

        context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &instructions,
                Some(&context.payer.pubkey()),
                &[&context.payer, &authority],
                context.last_blockhash,
            ))
            .await
            .unwrap();
    }
    // TODO: Set up the vote account with some stake.

    context.warp_to_epoch(next_epoch).unwrap();

    // TODO: Complete the test.
}
