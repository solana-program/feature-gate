#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use {
    mollusk_svm::Mollusk,
    solana_feature_gate_program::state::{FeatureBitMask, StagedFeatures, ValidatorSupportSignal},
    solana_sdk::{
        account::{Account, AccountSharedData},
        clock::Clock,
        feature::Feature,
        pubkey::Pubkey,
        rent::Rent,
        vote::state::{VoteInit, VoteState, VoteStateVersions},
    },
};

pub fn setup() -> Mollusk {
    Mollusk::new(
        &solana_feature_gate_program::id(),
        "solana_feature_gate_program",
    )
}

fn feature_rent() -> u64 {
    Rent::default().minimum_balance(Feature::size_of())
}

pub fn pending_feature_account() -> AccountSharedData {
    AccountSharedData::from(Account {
        lamports: feature_rent(),
        data: vec![
            0, // `None`
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
        owner: solana_feature_gate_program::id(),
        ..Account::default()
    })
}

pub fn active_feature_account() -> AccountSharedData {
    AccountSharedData::from(Account {
        lamports: feature_rent(),
        data: vec![
            1, // `Some`
            45, 0, 0, 0, 0, 0, 0, 0, // Random slot `u64`
        ],
        owner: solana_feature_gate_program::id(),
        ..Account::default()
    })
}

pub fn staged_features_account(feature_ids: &[(Pubkey, u64)]) -> AccountSharedData {
    let mut stage = StagedFeatures::default();
    for (i, (id, stake_support)) in feature_ids.iter().enumerate() {
        stage.features[i].feature_id = *id;
        stage.features[i].stake_support = *stake_support;
    }
    AccountSharedData::from(Account {
        lamports: 100_000_000,
        data: bytemuck::bytes_of(&stage).to_vec(),
        owner: solana_feature_gate_program::id(),
        ..Account::default()
    })
}

pub fn support_signal_account(signals: &[(u64, FeatureBitMask)]) -> AccountSharedData {
    let mut support_signals = ValidatorSupportSignal::default();
    for (slot, mask) in signals {
        support_signals.store_signal(*slot, *mask);
    }
    AccountSharedData::from(Account {
        lamports: 100_000_000,
        data: bytemuck::bytes_of(&support_signals).to_vec(),
        owner: solana_feature_gate_program::id(),
        ..Account::default()
    })
}

pub fn vote_account(authorized_voter: &Pubkey, clock: &Clock, _stake: u64) -> AccountSharedData {
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
    AccountSharedData::from(Account {
        lamports: 100_000_000,
        data,
        owner: solana_program::vote::program::id(),
        ..Account::default()
    })
}
