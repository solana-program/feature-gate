#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use {
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        pubkey::Pubkey,
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
