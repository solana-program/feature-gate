//! Feature Gate program compute unit benchmark testing.

use {
    mollusk::Mollusk,
    mollusk_bencher::{Bench, MolluskComputeUnitBencher},
    solana_feature_gate_program::instruction::revoke_pending_activation,
    solana_sdk::{account::AccountSharedData, feature::Feature, pubkey::Pubkey},
};

fn bench_revoke_feature() -> Bench {
    let feature = Pubkey::new_unique();
    let destination = Pubkey::new_unique();
    (
        "revoke_pending_activation".to_string(),
        revoke_pending_activation(&feature, &destination),
        vec![
            (
                feature,
                AccountSharedData::new_data(
                    42,
                    &Feature { activated_at: None },
                    &solana_sdk::feature::id(),
                )
                .unwrap(),
            ),
            (
                destination,
                AccountSharedData::new(0, 0, &solana_sdk::system_program::id()),
            ),
        ],
    )
}

fn main() {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");
    let mollusk = Mollusk::new(&solana_sdk::feature::id(), "solana_feature_gate_program");

    MolluskComputeUnitBencher::new(mollusk)
        .bench(bench_revoke_feature())
        .iterations(100)
        .must_pass(true)
        .out_dir("../target/benches")
        .execute();
}
