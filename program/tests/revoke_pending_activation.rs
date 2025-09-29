mod setup;

use {
    mollusk_svm::{program::keyed_account_for_system_program, result::Check},
    setup::{active_feature_account, pending_feature_account, setup},
    solana_feature_gate_program::{
        error::FeatureGateError, instruction::revoke_pending_activation,
    },
    solana_sdk::{
        account::{Account, WritableAccount},
        incinerator,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[test]
fn fail_feature_not_signer() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();

    let mut instruction = revoke_pending_activation(&feature);
    instruction.accounts[0].is_signer = false;

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (feature, pending_feature_account()),
            (incinerator::id(), Account::default()),
            keyed_account_for_system_program(),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );
}

#[test]
fn fail_feature_incorrect_owner() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();

    // Set up a feature account with incorrect owner.
    let mut feature_account = pending_feature_account();
    feature_account.set_owner(Pubkey::new_unique());

    mollusk.process_and_validate_instruction(
        &revoke_pending_activation(&feature),
        &[
            (feature, feature_account),
            (incinerator::id(), Account::default()),
            keyed_account_for_system_program(),
        ],
        &[Check::err(ProgramError::InvalidAccountOwner)],
    );
}

#[test]
fn fail_feature_invalid_data() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();

    // Set up a feature account with invalid data.
    let mut feature_account = pending_feature_account();
    feature_account.data = vec![2; 8];

    mollusk.process_and_validate_instruction(
        &revoke_pending_activation(&feature),
        &[
            (feature, feature_account),
            (incinerator::id(), Account::default()),
            keyed_account_for_system_program(),
        ],
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
fn fail_feature_already_activated() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();

    mollusk.process_and_validate_instruction(
        &revoke_pending_activation(&feature),
        &[
            (feature, active_feature_account()),
            (incinerator::id(), Account::default()),
            keyed_account_for_system_program(),
        ],
        &[Check::err(ProgramError::Custom(
            FeatureGateError::FeatureAlreadyActivated as u32,
        ))],
    );
}

#[test]
fn success() {
    let mollusk = setup();
    let feature = Pubkey::new_unique();

    mollusk.process_and_validate_instruction(
        &revoke_pending_activation(&feature),
        &[
            (feature, pending_feature_account()),
            (incinerator::id(), Account::default()),
            keyed_account_for_system_program(),
        ],
        &[
            Check::success(),
            Check::compute_units(2_724),
            // Confirm feature account was closed.
            Check::account(&feature).closed().build(),
        ],
    );
}
