#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use {
    mollusk_svm::Mollusk,
    solana_feature_gate_program::state::StagedFeatures,
    solana_sdk::{
        account::{Account, AccountSharedData},
        feature::Feature,
        pubkey::Pubkey,
        rent::Rent,
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

pub fn staged_features_account(feature_ids: &[Pubkey]) -> AccountSharedData {
    let mut stage = StagedFeatures::default();
    for (i, id) in feature_ids.iter().enumerate() {
        stage.features[i].feature_id = *id;
    }
    AccountSharedData::from(Account {
        lamports: 100_000_000,
        data: bytemuck::bytes_of(&stage).to_vec(),
        owner: solana_feature_gate_program::id(),
        ..Account::default()
    })
}
