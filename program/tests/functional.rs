#![cfg(feature = "test-sbf")]

use {
    mollusk_svm::{program::system_program, result::Check, Mollusk},
    solana_feature_gate_program::{
        error::FeatureGateError, instruction::revoke_pending_activation,
    },
    solana_sdk::{
        account::AccountSharedData, feature::Feature, incinerator, program_error::ProgramError,
        pubkey::Pubkey, rent::Rent,
    },
};

fn create_feature_account(feature: &Feature) -> AccountSharedData {
    let lamports = Rent::default().minimum_balance(Feature::size_of());
    AccountSharedData::new_data(lamports, feature, &solana_feature_gate_program::id()).unwrap()
}

#[test]
fn test_revoke_pending_activation() {
    let feature = Pubkey::new_unique();
    let mock_active_feature = Pubkey::new_unique();

    let mollusk = Mollusk::new(
        &solana_feature_gate_program::id(),
        "solana_feature_gate_program",
    );

    // Fail: feature not signer
    let mut instruction = revoke_pending_activation(&feature);
    instruction.accounts[0].is_signer = false;
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (
                feature,
                create_feature_account(&Feature { activated_at: None }),
            ),
            (incinerator::id(), AccountSharedData::default()),
            system_program(),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );

    // Fail: feature is already active
    mollusk.process_and_validate_instruction(
        &revoke_pending_activation(&mock_active_feature),
        &[
            (
                mock_active_feature,
                create_feature_account(&Feature {
                    activated_at: Some(500), // Random slot `u64`
                }),
            ),
            (incinerator::id(), AccountSharedData::default()),
            system_program(),
        ],
        &[Check::err(ProgramError::Custom(
            FeatureGateError::FeatureAlreadyActivated as u32,
        ))],
    );

    // Success: Revoke a feature activation
    mollusk.process_and_validate_instruction(
        &revoke_pending_activation(&feature),
        &[
            (
                feature,
                create_feature_account(&Feature { activated_at: None }),
            ),
            (incinerator::id(), AccountSharedData::default()),
            system_program(),
        ],
        &[
            Check::success(),
            // Confirm feature account was closed.
            Check::account(&feature).closed().build(),
        ],
    );
}
