#![allow(dead_code)]

use {
    mollusk_svm::Mollusk,
    solana_sdk::{account::Account, feature::Feature, rent::Rent},
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

pub fn pending_feature_account() -> Account {
    Account {
        lamports: feature_rent(),
        data: vec![
            0, // `None`
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
        owner: solana_feature_gate_program::id(),
        ..Account::default()
    }
}

pub fn active_feature_account() -> Account {
    Account {
        lamports: feature_rent(),
        data: vec![
            1, // `Some`
            45, 0, 0, 0, 0, 0, 0, 0, // Random slot `u64`
        ],
        owner: solana_feature_gate_program::id(),
        ..Account::default()
    }
}
