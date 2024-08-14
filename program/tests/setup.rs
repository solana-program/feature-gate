#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use {
    solana_feature_gate_program::state::{FeatureBitMask, StagedFeatures, ValidatorSupportSignal},
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        clock::{Clock, Epoch},
        pubkey::Pubkey,
        vote::state::{VoteInit, VoteState, VoteStateVersions},
    },
};

pub fn setup() -> ProgramTest {
    ProgramTest::new(
        "solana_feature_gate_program",
        solana_feature_gate_program::id(),
        processor!(solana_feature_gate_program::processor::process),
    )
}

pub fn setup_pending_feature(context: &mut ProgramTestContext, feature_id: &Pubkey) {
    context.set_account(
        feature_id,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![
                0, // `None`
                0, 0, 0, 0, 0, 0, 0, 0,
            ],
            owner: solana_feature_gate_program::id(),
            ..Account::default()
        }),
    );
}

pub fn setup_active_feature(context: &mut ProgramTestContext, feature_id: &Pubkey) {
    context.set_account(
        feature_id,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![
                1, // `Some`
                45, 0, 0, 0, 0, 0, 0, 0, // Random slot `u64`
            ],
            owner: solana_feature_gate_program::id(),
            ..Account::default()
        }),
    );
}

pub fn setup_stage(feature_ids: &[Pubkey]) -> StagedFeatures {
    let mut stage = StagedFeatures::default();
    for (i, id) in feature_ids.iter().enumerate() {
        stage.features[i].feature_id = *id;
    }
    stage
}

pub fn setup_support_signals(signals: &[(Epoch, FeatureBitMask)]) -> ValidatorSupportSignal {
    let mut support_signal = ValidatorSupportSignal::default();
    for (i, (epoch, signal)) in signals.iter().enumerate() {
        support_signal.signals[i].epoch = *epoch;
        support_signal.signals[i].signal = *signal;
    }
    support_signal
}

pub fn setup_staged_features_account(
    context: &mut ProgramTestContext,
    staged_features_address: &Pubkey,
    stage: StagedFeatures,
) {
    context.set_account(
        staged_features_address,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: bytemuck::bytes_of(&stage).to_vec(),
            owner: solana_feature_gate_program::id(),
            ..Account::default()
        }),
    );
}

pub fn setup_validator_support_signal_account(
    context: &mut ProgramTestContext,
    validator_support_signal_address: &Pubkey,
    signals: ValidatorSupportSignal,
) {
    context.set_account(
        validator_support_signal_address,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: bytemuck::bytes_of(&signals).to_vec(),
            owner: solana_feature_gate_program::id(),
            ..Account::default()
        }),
    );
}

pub fn setup_vote_account(
    context: &mut ProgramTestContext,
    vote_address: &Pubkey,
    authorized_voter: &Pubkey,
    clock: &Clock,
) {
    let data = {
        let vote_init = VoteInit {
            node_pubkey: Pubkey::new_unique(),
            authorized_voter: *authorized_voter,
            authorized_withdrawer: *authorized_voter,
            commission: 0,
        };
        let vote_state = VoteState::new(&vote_init, clock);
        let state = VoteStateVersions::new_current(vote_state);
        bincode::serialize(&state).unwrap()
    };
    context.set_account(
        vote_address,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data,
            owner: solana_program::vote::program::id(),
            ..Account::default()
        }),
    );
}

pub fn setup_vote_account_with_stake(
    _context: &mut ProgramTestContext,
    _vote_address: &Pubkey,
    _stake: u64,
) {
    todo!("Setup vote account with stake");
}
