//! Program state types.
use {
    bytemuck::{Pod, Zeroable},
    solana_program::{clock::Epoch, pubkey::Pubkey},
};

/// The seed prefix (`"staged_features"`) in bytes used to derive the address
/// of a staged features account.
/// Seeds: `"staged_features" + epoch`, where epoch is serialized to eight
/// little-endian bytes.
pub const SEED_PREFIX_STAGED_FEATURES: &[u8] = b"staged_features";

/// Derive the address of a staged features account.
pub fn get_staged_features_address(epoch: &Epoch) -> Pubkey {
    get_staged_features_address_and_bump_seed(epoch).0
}

/// Derive the address of a staged features account, with bump seed.
pub fn get_staged_features_address_and_bump_seed(epoch: &Epoch) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &collect_staged_features_seeds(&epoch.to_le_bytes()),
        &crate::id(),
    )
}

pub(crate) fn collect_staged_features_seeds(epoch_as_le: &[u8; 8]) -> [&[u8]; 2] {
    [SEED_PREFIX_STAGED_FEATURES, epoch_as_le]
}

#[allow(unused)] // For now.
pub(crate) fn collect_staged_features_signer_seeds<'a>(
    epoch_as_le: &'a [u8; 8],
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [SEED_PREFIX_STAGED_FEATURES, epoch_as_le, bump_seed]
}

/// The maximum number of features that can be staged per epoch.
const MAX_FEATURES: usize = 8;

/// A bitmask used to identify support for a set of features.
///
/// Each bit in the mask corresponds to a feature ID in the
/// staged features account. A bit set to 1 indicates support
/// for the corresponding feature.
///
/// Example:
///
/// ```text
/// Features = [A, B, C, D, E, F, G, H]
/// Bitmask = 170 = 10101010
/// Signalled support for features: A, C, E, G
/// ```
#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct FeatureBitMask(pub u8);

/// A Feature ID and its corresponding stake support, as signalled by
/// validators.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct FeatureStake {
    /// The ID of the feature.
    pub feature_id: Pubkey,
    /// The total stake supporting the feature.
    pub stake_support: u64,
}

/// Features staged for activation at the end of the epoch, with their
/// corresponding signalled stake support.
///
/// Supports a maximum of 8 features for any given epoch.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct StagedFeatures {
    /// Features staged for activation at the end of the epoch, with their
    /// corresponding signalled stake support.
    pub features: [FeatureStake; MAX_FEATURES],
}
