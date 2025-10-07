//! Feature Gate program compute unit benchmark testing.

use {
    mollusk_svm::{program::keyed_account_for_system_program, Mollusk},
    mollusk_svm_bencher::{Bench, MolluskComputeUnitBencher},
    solana_account::Account,
    solana_feature_gate_interface::Feature,
    solana_feature_gate_program::instruction::revoke_pending_activation,
    solana_pubkey::Pubkey,
    solana_sdk_ids::incinerator,
};

fn main() {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");
    let mollusk = Mollusk::new(
        &solana_sdk_ids::feature::id(),
        "solana_feature_gate_program",
    );

    let feature = Pubkey::new_unique();

    let bench: Bench = (
        "revoke_pending_activation",
        &revoke_pending_activation(&feature),
        &[
            (
                feature,
                Account::new_data_with_space(
                    42,
                    &Feature { activated_at: None },
                    9,
                    &solana_sdk_ids::feature::id(),
                )
                .unwrap(),
            ),
            (incinerator::id(), Account::default()),
            keyed_account_for_system_program(),
        ],
    );

    MolluskComputeUnitBencher::new(mollusk)
        .bench(bench)
        .must_pass(true)
        .out_dir("./benches")
        .execute();
}
