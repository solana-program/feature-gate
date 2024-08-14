#![cfg(feature = "test-sbf")]
#![allow(unused)] // Some of these tests require vote/stake account setup.

mod setup;

use {
    mollusk_svm::result::Check,
    setup::{
        active_feature_account, pending_feature_account, setup, staged_features_account,
        support_signal_account, vote_account,
    },
    solana_feature_gate_program::{
        error::FeatureGateError,
        instruction::signal_support_for_staged_features,
        state::{
            get_staged_features_address, get_validator_support_signal_address, FeatureBitMask,
            FeatureStake, StagedFeatures, ValidatorSupportSignal, MAX_FEATURES,
        },
    },
    solana_program::instruction::InstructionError,
    solana_sdk::{
        account::{Account, AccountSharedData},
        clock::Clock,
        program_error::ProgramError,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_program,
        transaction::{Transaction, TransactionError},
        vote::state::VoteStateVersions,
    },
    test_case::test_case,
};

#[test]
fn fail_authorized_voter_not_signer() {
    let mollusk = setup();
    let vote_account_pubkey = Pubkey::new_unique();
    let authorized_voter = Pubkey::new_unique();

    let staged_features_address = get_staged_features_address(&mollusk.sysvars.clock.epoch);
    let validator_support_signal_address =
        get_validator_support_signal_address(&vote_account_pubkey);

    let mut instruction = signal_support_for_staged_features(
        &staged_features_address,
        &validator_support_signal_address,
        &vote_account_pubkey,
        &authorized_voter,
        FeatureBitMask(0),
    );
    instruction.accounts[3].is_signer = false; // Vote account not signer.

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (staged_features_address, AccountSharedData::default()),
            (
                validator_support_signal_address,
                AccountSharedData::default(),
            ),
            (vote_account_pubkey, AccountSharedData::default()),
            (authorized_voter, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );
}

#[test]
fn fail_vote_account_incorrect_owner() {
    let mollusk = setup();
    let vote_account_pubkey = Pubkey::new_unique();
    let authorized_voter = Pubkey::new_unique();

    let staged_features_address = get_staged_features_address(&mollusk.sysvars.clock.epoch);
    let validator_support_signal_address =
        get_validator_support_signal_address(&vote_account_pubkey);

    // Set up a vote account with incorrect owner.
    let vote_account = {
        let space = VoteStateVersions::vote_state_size_of(true);
        AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![8; space],
            owner: Pubkey::new_unique(), // Incorrect owner.
            ..Account::default()
        })
    };

    mollusk.process_and_validate_instruction(
        &signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account_pubkey,
            &authorized_voter,
            FeatureBitMask(0),
        ),
        &[
            (staged_features_address, AccountSharedData::default()),
            (
                validator_support_signal_address,
                AccountSharedData::default(),
            ),
            (vote_account_pubkey, vote_account),
            (authorized_voter, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::InvalidAccountOwner)],
    );
}

#[test]
fn fail_vote_account_invalid_state() {
    let mollusk = setup();
    let vote_account_pubkey = Pubkey::new_unique();
    let authorized_voter = Pubkey::new_unique();

    let staged_features_address = get_staged_features_address(&mollusk.sysvars.clock.epoch);
    let validator_support_signal_address =
        get_validator_support_signal_address(&vote_account_pubkey);

    // Set up a vote account with invalid state.
    let vote_account = AccountSharedData::from(Account {
        lamports: 100_000_000,
        data: vec![8; 32],
        owner: solana_program::vote::program::id(),
        ..Account::default()
    });

    mollusk.process_and_validate_instruction(
        &signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account_pubkey,
            &authorized_voter,
            FeatureBitMask(0),
        ),
        &[
            (staged_features_address, AccountSharedData::default()),
            (
                validator_support_signal_address,
                AccountSharedData::default(),
            ),
            (vote_account_pubkey, vote_account),
            (authorized_voter, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
fn fail_incorrect_authorized_voter() {
    let mollusk = setup();
    let vote_account_pubkey = Pubkey::new_unique();
    let authorized_voter = Pubkey::new_unique();

    let clock = &mollusk.sysvars.clock;
    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address =
        get_validator_support_signal_address(&vote_account_pubkey);

    // Set up a vote account with incorrect authorized voter.
    let vote_account = vote_account(
        &Pubkey::new_unique(), // Incorrect authorized voter.
        clock,
        /* stake */ 0,
    );

    mollusk.process_and_validate_instruction(
        &signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account_pubkey,
            &authorized_voter,
            FeatureBitMask(0),
        ),
        &[
            (staged_features_address, AccountSharedData::default()),
            (
                validator_support_signal_address,
                AccountSharedData::default(),
            ),
            (vote_account_pubkey, vote_account),
            (authorized_voter, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::IncorrectAuthority)],
    );
}

#[test]
fn fail_staged_features_incorrect_address() {
    let mollusk = setup();
    let vote_account_pubkey = Pubkey::new_unique();
    let authorized_voter = Pubkey::new_unique();

    let clock = &mollusk.sysvars.clock;
    let staged_features_address = Pubkey::new_unique(); // Incorrect address.
    let validator_support_signal_address =
        get_validator_support_signal_address(&vote_account_pubkey);

    let staged_features_account = staged_features_account(&[]);
    let vote_account = vote_account(&authorized_voter, clock, /* stake */ 0);

    mollusk.process_and_validate_instruction(
        &signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account_pubkey,
            &authorized_voter,
            FeatureBitMask(0),
        ),
        &[
            (staged_features_address, staged_features_account),
            (
                validator_support_signal_address,
                AccountSharedData::default(),
            ),
            (vote_account_pubkey, vote_account),
            (authorized_voter, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::Custom(
            FeatureGateError::IncorrectStagedFeaturesAddress as u32,
        ))],
    );
}

#[test]
fn fail_staged_features_incorrect_owner() {
    let mollusk = setup();
    let vote_account_pubkey = Pubkey::new_unique();
    let authorized_voter = Pubkey::new_unique();

    let clock = &mollusk.sysvars.clock;
    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address =
        get_validator_support_signal_address(&vote_account_pubkey);

    let vote_account = vote_account(&authorized_voter, clock, /* stake */ 0);

    // Set up a staged features account with the wrong owner.
    let staged_features_account = AccountSharedData::from(Account {
        lamports: 100_000_000,
        data: vec![0; std::mem::size_of::<StagedFeatures>()],
        owner: system_program::id(),
        ..Account::default()
    });

    mollusk.process_and_validate_instruction(
        &signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account_pubkey,
            &authorized_voter,
            FeatureBitMask(0),
        ),
        &[
            (staged_features_address, staged_features_account),
            (
                validator_support_signal_address,
                AccountSharedData::default(),
            ),
            (vote_account_pubkey, vote_account),
            (authorized_voter, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::IncorrectProgramId)],
    );
}

#[test]
fn fail_staged_features_invalid_data() {
    let mollusk = setup();
    let vote_account_pubkey = Pubkey::new_unique();
    let authorized_voter = Pubkey::new_unique();

    let clock = &mollusk.sysvars.clock;
    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address =
        get_validator_support_signal_address(&vote_account_pubkey);

    let vote_account = vote_account(&authorized_voter, clock, /* stake */ 0);

    // Set up a staged features account with invalid data.
    let staged_features_account = AccountSharedData::from(Account {
        lamports: 100_000_000,
        data: vec![8; std::mem::size_of::<StagedFeatures>().saturating_add(1)],
        owner: solana_feature_gate_program::id(),
        ..Account::default()
    });

    mollusk.process_and_validate_instruction(
        &signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account_pubkey,
            &authorized_voter,
            FeatureBitMask(0),
        ),
        &[
            (staged_features_address, staged_features_account),
            (
                validator_support_signal_address,
                AccountSharedData::default(),
            ),
            (vote_account_pubkey, vote_account),
            (authorized_voter, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[cfg(skip)] // Requires vote/stake account setup.
#[test]
fn fail_validator_support_signal_incorrect_address() {
    let mollusk = setup();
    let vote_account_pubkey = Pubkey::new_unique();
    let authorized_voter = Pubkey::new_unique();

    let signal = FeatureBitMask(0);

    let clock = &mollusk.sysvars.clock;
    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address = Pubkey::new_unique(); // Incorrect address.

    let staged_features_account = staged_features_account(&[]);
    let support_signal_account = support_signal_account(&[(clock.epoch, signal.clone())]);
    let vote_account = vote_account(&authorized_voter, clock, /* stake */ 0);

    mollusk.process_and_validate_instruction(
        &signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account_pubkey,
            &authorized_voter,
            FeatureBitMask(0),
        ),
        &[
            (staged_features_address, staged_features_account),
            (validator_support_signal_address, support_signal_account),
            (vote_account_pubkey, vote_account),
            (authorized_voter, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::Custom(
            FeatureGateError::IncorrectValidatorSupportSignalAddress as u32,
        ))],
    );
}

#[cfg(skip)] // Requires vote/stake account setup.
#[test]
fn fail_validator_support_signal_invalid_data() {
    let mollusk = setup();
    let vote_account_pubkey = Pubkey::new_unique();
    let authorized_voter = Pubkey::new_unique();

    let signal = FeatureBitMask(0);

    let clock = &mollusk.sysvars.clock;
    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address =
        get_validator_support_signal_address(&vote_account_pubkey);

    let staged_features_account = staged_features_account(&[]);
    let vote_account = vote_account(&authorized_voter, clock, /* stake */ 100_000_000);

    // Set up a validator support signal account with invalid data.
    let support_signal_account = AccountSharedData::from(Account {
        lamports: 100_000_000,
        data: vec![8; std::mem::size_of::<FeatureBitMask>().saturating_add(1)],
        owner: solana_feature_gate_program::id(),
        ..Account::default()
    });

    mollusk.process_and_validate_instruction(
        &signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account_pubkey,
            &authorized_voter,
            FeatureBitMask(0),
        ),
        &[
            (staged_features_address, staged_features_account),
            (validator_support_signal_address, support_signal_account),
            (vote_account_pubkey, vote_account),
            (authorized_voter, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

struct Case {
    previous_signal: Option<FeatureBitMask>,
    signal: FeatureBitMask,
    stake: u64,
}

#[test_case(Case {
    previous_signal: None,
    signal: FeatureBitMask(0),
    stake: 100_000_000,
})]
fn success(case: Case) {
    let mollusk = setup();
    let vote_account_pubkey = Pubkey::new_unique();
    let authorized_voter = Pubkey::new_unique();

    let clock = &mollusk.sysvars.clock;
    let staged_features_address = get_staged_features_address(&clock.epoch);
    let validator_support_signal_address =
        get_validator_support_signal_address(&vote_account_pubkey);

    let features = (0..MAX_FEATURES)
        .map(|i| (Pubkey::new_unique(), 10_000_000_000u64))
        .collect::<Vec<_>>();

    let Case {
        previous_signal,
        signal,
        stake,
    } = case;

    let staged_features_account = staged_features_account(&features);
    let support_signal_account = if let Some(previous_signal) = previous_signal {
        support_signal_account(&[(clock.epoch, previous_signal)])
    } else {
        // Fund the account with enough rent-exempt lamports.
        let rent = &mollusk.sysvars.rent;
        let space = std::mem::size_of::<ValidatorSupportSignal>();
        let lamports = rent.minimum_balance(space);
        AccountSharedData::new(lamports, space, &solana_feature_gate_program::id())
    };
    let vote_account = vote_account(&authorized_voter, clock, /* stake */ stake);

    mollusk.process_and_validate_instruction(
        &signal_support_for_staged_features(
            &staged_features_address,
            &validator_support_signal_address,
            &vote_account_pubkey,
            &authorized_voter,
            signal,
        ),
        &[
            (staged_features_address, staged_features_account),
            (validator_support_signal_address, support_signal_account),
            (vote_account_pubkey, vote_account),
            (authorized_voter, AccountSharedData::default()),
        ],
        &[
            Check::success(),
            // TODO: Checks
        ],
    );

    // TODO: Check the resulting staged features account...
}
