#![cfg(feature = "test-sbf")]
#![allow(unused)] // Some of these tests require vote/stake account setup.

mod setup;

use {
    setup::{
        setup, setup_stage, setup_staged_features_account, setup_support_signals,
        setup_validator_support_signal_account, setup_vote_account, setup_vote_account_with_stake,
    },
    solana_feature_gate_program::{
        error::FeatureGateError,
        instruction::signal_support_for_staged_features,
        state::{
            get_staged_features_address, get_validator_support_signal_address, FeatureBitMask,
            FeatureStake, StagedFeatures,
        },
    },
    solana_program::instruction::InstructionError,
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        clock::Clock,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_program,
        transaction::{Transaction, TransactionError},
        vote::state::VoteStateVersions,
    },
    test_case::test_case,
};

#[tokio::test]
async fn fail_authorized_voter_not_signer() {
    let vote_account = Pubkey::new_unique();
    let authorized_voter = Keypair::new();

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address = get_validator_support_signal_address(&vote_account);

    let mut instruction = signal_support_for_staged_features(
        &staged_features_address,
        &validator_support_signal_address,
        &vote_account,
        &authorized_voter.pubkey(),
        FeatureBitMask(0),
        /* init */ true,
    );
    instruction.accounts[3].is_signer = false; // Vote account not signer.

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
async fn fail_vote_account_incorrect_owner() {
    let vote_account = Pubkey::new_unique();
    let authorized_voter = Keypair::new();

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address = get_validator_support_signal_address(&vote_account);

    // Set up a vote account with incorrect owner.
    {
        let space = VoteStateVersions::vote_state_size_of(true);
        context.set_account(
            &vote_account,
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![8; space],
                owner: Pubkey::new_unique(), // Incorrect owner.
                ..Account::default()
            }),
        );
    }

    let transaction = Transaction::new_signed_with_payer(
        &[signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account,
            &authorized_voter.pubkey(),
            FeatureBitMask(0),
            /* init */ true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authorized_voter],
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
async fn fail_vote_account_invalid_state() {
    let vote_account = Pubkey::new_unique();
    let authorized_voter = Keypair::new();

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address = get_validator_support_signal_address(&vote_account);

    // Set up a vote account with invalid state.
    {
        context.set_account(
            &vote_account,
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![8; 32],
                owner: solana_program::vote::program::id(),
                ..Account::default()
            }),
        );
    }

    let transaction = Transaction::new_signed_with_payer(
        &[signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account,
            &authorized_voter.pubkey(),
            FeatureBitMask(0),
            /* init */ true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authorized_voter],
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
async fn fail_incorrect_authorized_voter() {
    let vote_account = Pubkey::new_unique();
    let authorized_voter = Keypair::new();

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address = get_validator_support_signal_address(&vote_account);

    // Set up a vote account with incorrect authorized voter.
    setup_vote_account(
        &mut context,
        &vote_account,
        &Pubkey::new_unique(), // Incorrect authorized voter.
        &clock,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account,
            &authorized_voter.pubkey(),
            FeatureBitMask(0),
            /* init */ true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authorized_voter],
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
async fn fail_staged_features_incorrect_address() {
    let vote_account = Pubkey::new_unique();
    let authorized_voter = Keypair::new();

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    let staged_features_address = Pubkey::new_unique(); // Incorrect address.
    let validator_support_signal_address = get_validator_support_signal_address(&vote_account);

    setup_staged_features_account(&mut context, &staged_features_address, setup_stage(&[]));
    setup_vote_account(
        &mut context,
        &vote_account,
        &authorized_voter.pubkey(),
        &clock,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account,
            &authorized_voter.pubkey(),
            FeatureBitMask(0),
            /* init */ true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authorized_voter],
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
async fn fail_staged_features_incorrect_owner() {
    let vote_account = Pubkey::new_unique();
    let authorized_voter = Keypair::new();

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address = get_validator_support_signal_address(&vote_account);

    setup_vote_account(
        &mut context,
        &vote_account,
        &authorized_voter.pubkey(),
        &clock,
    );

    // Set up a staged features account with the wrong owner.
    {
        context.set_account(
            &staged_features_address,
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![0; std::mem::size_of::<StagedFeatures>()],
                owner: system_program::id(),
                ..Account::default()
            }),
        );
    }

    let transaction = Transaction::new_signed_with_payer(
        &[signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account,
            &authorized_voter.pubkey(),
            FeatureBitMask(0),
            /* init */ true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authorized_voter],
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
        TransactionError::InstructionError(0, InstructionError::IncorrectProgramId)
    );
}

#[tokio::test]
async fn fail_staged_features_invalid_data() {
    let vote_account = Pubkey::new_unique();
    let authorized_voter = Keypair::new();

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address = get_validator_support_signal_address(&vote_account);

    setup_vote_account(
        &mut context,
        &vote_account,
        &authorized_voter.pubkey(),
        &clock,
    );

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
            &validator_support_signal_address,
            &vote_account,
            &authorized_voter.pubkey(),
            FeatureBitMask(0),
            /* init */ true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authorized_voter],
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

#[cfg(skip)] // Requires vote/stake account setup.
#[test_case(true; "initialized")]
#[test_case(false; "not initialized")]
#[tokio::test]
async fn fail_validator_support_signal_incorrect_address(initialized: bool) {
    let vote_account = Pubkey::new_unique();
    let authorized_voter = Keypair::new();

    let signal = FeatureBitMask(0);

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address = Pubkey::new_unique(); // Incorrect address.

    setup_vote_account_with_stake(&mut context, &vote_account, 100_000_000);
    setup_staged_features_account(&mut context, &staged_features_address, setup_stage(&[]));
    setup_vote_account(
        &mut context,
        &vote_account,
        &authorized_voter.pubkey(),
        &clock,
    );

    // If `initialized: true`, set up a validator support signal account ahead
    // of time.
    if initialized {
        setup_validator_support_signal_account(
            &mut context,
            &validator_support_signal_address,
            setup_support_signals(&[(clock.epoch, signal.clone())]),
        );
    }

    let transaction = Transaction::new_signed_with_payer(
        &[signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account,
            &authorized_voter.pubkey(),
            signal,
            /* init */ true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authorized_voter],
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
            InstructionError::Custom(
                FeatureGateError::IncorrectValidatorSupportSignalAddress as u32
            )
        )
    );
}

#[cfg(skip)] // Requires vote/stake account setup.
#[tokio::test]
async fn fail_validator_support_signal_invalid_data() {
    let vote_account = Pubkey::new_unique();
    let authorized_voter = Keypair::new();

    let signal = FeatureBitMask(0);

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address = get_validator_support_signal_address(&vote_account);

    setup_vote_account_with_stake(&mut context, &vote_account, 100_000_000);
    setup_staged_features_account(&mut context, &staged_features_address, setup_stage(&[]));
    setup_vote_account(
        &mut context,
        &vote_account,
        &authorized_voter.pubkey(),
        &clock,
    );

    // Set up a validator support signal account with invalid data.
    {
        context.set_account(
            &validator_support_signal_address,
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![8; std::mem::size_of::<FeatureBitMask>().saturating_add(1)],
                owner: solana_feature_gate_program::id(),
                ..Account::default()
            }),
        );
    }

    let transaction = Transaction::new_signed_with_payer(
        &[signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account,
            &authorized_voter.pubkey(),
            signal,
            /* init */ true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authorized_voter],
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

struct SuccessTestCase {
    previous_signal: Option<FeatureBitMask>,
    signal: FeatureBitMask,
    stake: u64,
}

#[cfg(skip)] // Requires vote/stake account setup.
#[test_case(SuccessTestCase {
    previous_signal: None,
    signal: FeatureBitMask(0),
    stake: 100_000_000,
})]
#[tokio::test]
async fn success(case: SuccessTestCase) {
    let vote_account = Pubkey::new_unique();
    let authorized_voter = Keypair::new();

    let signal = FeatureBitMask(0);

    let mut context = setup().start_with_context().await;
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();

    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address = get_validator_support_signal_address(&vote_account);

    setup_vote_account_with_stake(&mut context, &vote_account, case.stake);
    setup_vote_account(
        &mut context,
        &vote_account,
        &authorized_voter.pubkey(),
        &clock,
    );

    // Set up the staged features account with some beginning stake support,
    // then add the validator's previous bitmask (if any).
    {
        let mut stage = StagedFeatures {
            features: [FeatureStake {
                feature_id: Pubkey::default(),
                stake_support: 0,
            }; 8],
        };
        for feature in stage.features.iter_mut() {
            feature.feature_id = Pubkey::new_unique();
            feature.stake_support = 10_000_000_000;
        }
        if let Some(previous_signal) = case.previous_signal {
            stage.add_stake_support(&previous_signal, case.stake);
        }
        setup_staged_features_account(&mut context, &staged_features_address, stage);
    }

    // If there is a previous signal, set up the validator support signal
    // account with the previous signal.
    if let Some(previous_signal) = case.previous_signal {
        setup_validator_support_signal_account(
            &mut context,
            &validator_support_signal_address,
            setup_support_signals(&[(clock.epoch, previous_signal.clone())]),
        );
    }

    // Get the initial staged features account, for checks later.
    let initial_staged_features_account = context
        .banks_client
        .get_account(staged_features_address)
        .await
        .unwrap()
        .unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account,
            &authorized_voter.pubkey(),
            signal,
            /* init */ true,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authorized_voter],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Check the resulting staged features account.
    let resulting_staged_features_account = context
        .banks_client
        .get_account(staged_features_address)
        .await
        .unwrap()
        .unwrap();

    // Ensure that for each feature in the stage, any previous signal was
    // deducted, and then any current signal was added.
    let initial_stage =
        bytemuck::from_bytes::<StagedFeatures>(&initial_staged_features_account.data);
    let resulting_stage =
        bytemuck::from_bytes::<StagedFeatures>(&resulting_staged_features_account.data);
    let previous_signal = case
        .previous_signal
        .map_or([false; 8], |signal| (&signal).into());
    let current_signal: [bool; 8] = (&signal).into();
    for (i, (initial, resulting)) in initial_stage
        .features
        .iter()
        .zip(resulting_stage.features.iter())
        .enumerate()
    {
        #[allow(clippy::arithmetic_side_effects)]
        assert_eq!(
            resulting.stake_support,
            initial.stake_support - (previous_signal[i] as u64 * case.stake)
                + (current_signal[i] as u64 * case.stake)
        )
    }
}
