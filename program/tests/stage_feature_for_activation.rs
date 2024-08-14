#![cfg(feature = "test-sbf")]

mod setup;

use {
    mollusk_svm::result::Check,
    setup::{active_feature_account, pending_feature_account, setup, staged_features_account},
    solana_feature_gate_program::{
        error::FeatureGateError,
        instruction::stage_feature_for_activation,
        state::{get_staged_features_address, FeatureStake, StagedFeatures, MAX_FEATURES},
    },
    solana_sdk::{
        account::{Account, AccountSharedData},
        feature::Feature,
        program_error::ProgramError,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        signer::SeedDerivable,
    },
    test_case::test_case,
};

// Simply a keypair created with seed `[0; 32]`, for now.
fn stage_authority_pubkey() -> Pubkey {
    Keypair::from_seed(&[0u8; 32]).unwrap().pubkey()
}

#[test]
fn fail_incorrect_authority() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();
    let authority = Pubkey::new_unique(); // Incorrect authority.

    let staged_features_address =
        get_staged_features_address(&mollusk.sysvars.clock.epoch.saturating_add(1));

    mollusk.process_and_validate_instruction(
        &stage_feature_for_activation(&feature, &staged_features_address, &authority),
        &[
            (feature, AccountSharedData::default()),
            (staged_features_address, AccountSharedData::default()),
            (authority, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::IncorrectAuthority)],
    );
}

#[test]
fn fail_authority_not_signer() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();
    let authority = stage_authority_pubkey();

    let staged_features_address =
        get_staged_features_address(&mollusk.sysvars.clock.epoch.saturating_add(1));

    let mut instruction =
        stage_feature_for_activation(&feature, &staged_features_address, &authority);
    instruction.accounts[2].is_signer = false; // Authority not signer.

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (feature, AccountSharedData::default()),
            (staged_features_address, AccountSharedData::default()),
            (authority, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );
}

#[test]
fn fail_feature_incorrect_owner() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();
    let authority = stage_authority_pubkey();

    let staged_features_address =
        get_staged_features_address(&mollusk.sysvars.clock.epoch.saturating_add(1));

    // Set up a feature account with incorrect owner.
    let feature_account = AccountSharedData::from(Account {
        lamports: 100_000_000,
        data: vec![0; Feature::size_of()],
        owner: Pubkey::new_unique(), // Incorrect owner.
        ..Account::default()
    });

    mollusk.process_and_validate_instruction(
        &stage_feature_for_activation(&feature, &staged_features_address, &authority),
        &[
            (feature, feature_account),
            (staged_features_address, AccountSharedData::default()),
            (authority, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::InvalidAccountOwner)],
    );
}

#[test]
fn fail_feature_invalid_data() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();
    let authority = stage_authority_pubkey();

    let staged_features_address =
        get_staged_features_address(&mollusk.sysvars.clock.epoch.saturating_add(1));

    // Set up a feature account with invalid data.
    let feature_account = AccountSharedData::from(Account {
        lamports: 100_000_000,
        data: vec![8; Feature::size_of()],
        owner: solana_feature_gate_program::id(),
        ..Account::default()
    });

    mollusk.process_and_validate_instruction(
        &stage_feature_for_activation(&feature, &staged_features_address, &authority),
        &[
            (feature, feature_account),
            (staged_features_address, AccountSharedData::default()),
            (authority, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
fn fail_feature_already_activated() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();
    let authority = stage_authority_pubkey();

    let staged_features_address =
        get_staged_features_address(&mollusk.sysvars.clock.epoch.saturating_add(1));

    // Set up an active feature account.
    let feature_account = active_feature_account();

    mollusk.process_and_validate_instruction(
        &stage_feature_for_activation(&feature, &staged_features_address, &authority),
        &[
            (feature, feature_account),
            (staged_features_address, AccountSharedData::default()),
            (authority, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::Custom(
            FeatureGateError::FeatureAlreadyActivated as u32,
        ))],
    );
}

#[test]
fn fail_staged_features_incorrect_address() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();
    let authority = stage_authority_pubkey();

    let staged_features_address = Pubkey::new_unique(); // Incorrect address.

    let feature_account = pending_feature_account();
    let stage_account = staged_features_account(&[]);

    mollusk.process_and_validate_instruction(
        &stage_feature_for_activation(&feature, &staged_features_address, &authority),
        &[
            (feature, feature_account),
            (staged_features_address, stage_account),
            (authority, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::Custom(
            FeatureGateError::IncorrectStagedFeaturesAddress as u32,
        ))],
    );
}

#[test]
fn fail_staged_features_incorrect_epoch() {
    let mut mollusk = setup();
    let feature = Pubkey::new_unique();
    let authority = stage_authority_pubkey();

    let upcoming_epoch = mollusk.sysvars.clock.epoch.saturating_add(1);
    let staged_features_address = get_staged_features_address(&upcoming_epoch);

    // Warp to the next epoch, making it the wrong staged features account.
    let warp_slot = mollusk
        .sysvars
        .epoch_schedule
        .get_first_slot_in_epoch(upcoming_epoch);
    mollusk.warp_to_slot(warp_slot);

    let feature_account = pending_feature_account();
    let stage_account = staged_features_account(&[]);

    mollusk.process_and_validate_instruction(
        &stage_feature_for_activation(&feature, &staged_features_address, &authority),
        &[
            (feature, feature_account),
            (staged_features_address, stage_account),
            (authority, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::Custom(
            FeatureGateError::IncorrectStagedFeaturesAddress as u32,
        ))],
    );
}

#[test]
fn fail_staged_features_invalid_data() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();
    let authority = stage_authority_pubkey();

    let upcoming_epoch = mollusk.sysvars.clock.epoch.saturating_add(1);
    let staged_features_address = get_staged_features_address(&upcoming_epoch);

    let feature_account = pending_feature_account();

    // Set up a staged features account with invalid data.
    let stage_account = AccountSharedData::from(Account {
        lamports: 100_000_000,
        data: vec![8; std::mem::size_of::<StagedFeatures>().saturating_add(1)],
        owner: solana_feature_gate_program::id(),
        ..Account::default()
    });

    mollusk.process_and_validate_instruction(
        &stage_feature_for_activation(&feature, &staged_features_address, &authority),
        &[
            (feature, feature_account),
            (staged_features_address, stage_account),
            (authority, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
fn fail_feature_already_staged() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();
    let authority = stage_authority_pubkey();

    let upcoming_epoch = mollusk.sysvars.clock.epoch.saturating_add(1);
    let staged_features_address = get_staged_features_address(&upcoming_epoch);

    let feature_account = pending_feature_account();
    let stage_account = staged_features_account(
        &[(feature, 100_000_000)], // Feature already staged.
    );

    mollusk.process_and_validate_instruction(
        &stage_feature_for_activation(&feature, &staged_features_address, &authority),
        &[
            (feature, feature_account),
            (staged_features_address, stage_account),
            (authority, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::Custom(
            FeatureGateError::FeatureAlreadyStaged as u32,
        ))],
    );
}

#[test]
fn fail_feature_stage_full() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();
    let authority = stage_authority_pubkey();

    let upcoming_epoch = mollusk.sysvars.clock.epoch.saturating_add(1);
    let staged_features_address = get_staged_features_address(&upcoming_epoch);

    let feature_account = pending_feature_account();
    let stage_account = staged_features_account(
        &[(Pubkey::new_unique(), 100_000_000); MAX_FEATURES], // Stage full.
    );

    mollusk.process_and_validate_instruction(
        &stage_feature_for_activation(&feature, &staged_features_address, &authority),
        &[
            (feature, feature_account),
            (staged_features_address, stage_account),
            (authority, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::Custom(
            FeatureGateError::FeatureStageFull as u32,
        ))],
    );
}

enum Case {
    Initialized { features: Vec<(Pubkey, u64)> },
    NotInitialized,
}

#[test_case(Case::Initialized {
    features: vec![]
}; "initialized with no keys")]
#[test_case(
    Case::Initialized { features: vec![
        (Pubkey::new_unique(), 100_000_000),
        (Pubkey::new_unique(), 200_000_000),
    ]
}; "initialized with 2 keys")]
#[test_case(
    Case::Initialized { features: vec![
        (Pubkey::new_unique(), 100_000_000),
        (Pubkey::new_unique(), 200_000_000),
        (Pubkey::new_unique(), 300_000_000),
        (Pubkey::new_unique(), 400_000_000),
    ]
}; "initialized with 4 keys")]
#[test_case(
    Case::Initialized { features: vec![
        (Pubkey::new_unique(), 100_000_000),
        (Pubkey::new_unique(), 200_000_000),
        (Pubkey::new_unique(), 300_000_000),
        (Pubkey::new_unique(), 400_000_000),
        (Pubkey::new_unique(), 500_000_000),
        (Pubkey::new_unique(), 600_000_000),
        (Pubkey::new_unique(), 700_000_000),
    ]
}; "initialized with 7 keys")]
#[test_case(Case::NotInitialized; "not initialized")]
fn success(case: Case) {
    let mollusk = setup();
    let feature = Pubkey::new_unique();
    let authority = stage_authority_pubkey();

    let upcoming_epoch = mollusk.sysvars.clock.epoch.saturating_add(1);
    let staged_features_address = get_staged_features_address(&upcoming_epoch);

    let feature_account = pending_feature_account();

    let process_instruction = |stage_account: AccountSharedData,
                               initial_features: &[(Pubkey, u64)]| {
        // The stage should have our new feature.
        let mut expected_feature_stage = [FeatureStake::default(); MAX_FEATURES];
        initial_features
            .iter()
            .chain(std::iter::once(&(feature, 0)))
            .enumerate()
            .for_each(|(i, (id, stake_support))| {
                expected_feature_stage[i].feature_id = *id;
                expected_feature_stage[i].stake_support = *stake_support;
            });

        mollusk.process_and_validate_instruction(
            &stage_feature_for_activation(&feature, &staged_features_address, &authority),
            &[
                (feature, feature_account),
                (staged_features_address, stage_account),
                (authority, AccountSharedData::default()),
            ],
            &[
                Check::success(),
                Check::account(&staged_features_address)
                    .data(bytemuck::bytes_of(&StagedFeatures {
                        features: expected_feature_stage,
                    }))
                    .build(),
            ],
        )
    };

    match case {
        Case::Initialized { features } => {
            let stage_account = staged_features_account(&features);

            process_instruction(stage_account, &features);
        }
        Case::NotInitialized => {
            let stage_account = {
                // Fund the account with enough rent-exempt lamports.
                let rent = &mollusk.sysvars.rent;
                let space = std::mem::size_of::<StagedFeatures>();
                let lamports = rent.minimum_balance(space);
                AccountSharedData::new(lamports, space, &solana_feature_gate_program::id())
            };

            process_instruction(stage_account, &[]);
        }
    }
}
