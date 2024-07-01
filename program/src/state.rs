//! Program state types.
use {
    crate::error::FeatureGateError,
    bytemuck::{Pod, Zeroable},
    solana_program::{clock::Epoch, program_error::ProgramError, pubkey::Pubkey},
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

impl From<&[bool; MAX_FEATURES]> for FeatureBitMask {
    fn from(signals: &[bool; MAX_FEATURES]) -> Self {
        let mut mask = 0;
        for i in 0..MAX_FEATURES {
            if signals[MAX_FEATURES - 1 - i] {
                mask |= 1 << i;
            }
        }
        Self(mask)
    }
}

impl From<&FeatureBitMask> for [bool; MAX_FEATURES] {
    fn from(mask: &FeatureBitMask) -> [bool; MAX_FEATURES] {
        let mut signals = [false; MAX_FEATURES];
        for i in 0..MAX_FEATURES {
            signals[MAX_FEATURES - 1 - i] = mask.0 & (1 << i) != 0;
        }
        signals
    }
}

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

impl FeatureStake {
    fn is_initialized(&self) -> bool {
        self.feature_id != Pubkey::default()
    }

    fn add_stake_support(&mut self, stake: u64) {
        self.stake_support += stake;
    }
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

impl StagedFeatures {
    /// Stage a feature for activation by adding it to the array.
    pub fn stage(&mut self, feature_id: &Pubkey) -> Result<(), ProgramError> {
        if self.features.iter().any(|f| &f.feature_id == feature_id) {
            return Err(FeatureGateError::FeatureAlreadyStaged.into());
        }
        if let Some(slot) = self.features.iter_mut().position(|f| !f.is_initialized()) {
            self.features[slot].feature_id = *feature_id;
            return Ok(());
        }
        Err(FeatureGateError::FeatureStageFull.into())
    }

    /// Add stake support for features given a set of signals.
    pub fn add_stake_support(
        &mut self,
        bitmask: &FeatureBitMask,
        _total_epoch_stake: u64,
        vote_account_epoch_stake: u64,
    ) {
        <[bool; MAX_FEATURES]>::from(bitmask)
            .iter()
            .enumerate()
            .filter(|(_, signal)| **signal)
            .for_each(|(i, _)| {
                if let Some(feature) = self.features.get_mut(i) {
                    if feature.is_initialized() {
                        feature.add_stake_support(vote_account_epoch_stake);
                    }
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_stage(feature_ids: &[Pubkey]) -> StagedFeatures {
        let mut features = [FeatureStake {
            feature_id: Pubkey::default(),
            stake_support: 0,
        }; MAX_FEATURES];

        for (i, id) in feature_ids.iter().enumerate() {
            features[i].feature_id = *id;
        }

        StagedFeatures { features }
    }

    #[test]
    fn test_stage_fail_feature_already_staged() {
        let feature_id = Pubkey::new_unique();

        let mut stage = setup_stage(&[feature_id]);

        assert_eq!(
            stage.stage(&feature_id).unwrap_err(),
            FeatureGateError::FeatureAlreadyStaged.into()
        );
    }

    #[test]
    fn test_stage_fail_stage_full() {
        let feature_id = Pubkey::new_unique();

        let staged_features = vec![Pubkey::new_unique(); MAX_FEATURES];
        let mut stage = setup_stage(&staged_features);

        assert_eq!(
            stage.stage(&feature_id).unwrap_err(),
            FeatureGateError::FeatureStageFull.into()
        );
    }

    #[test]
    fn test_stage_success() {
        let feature_id = Pubkey::new_unique();

        // Works with an empty stage.
        let mut stage = setup_stage(&[]);
        assert_eq!(stage.stage(&feature_id), Ok(()));
        assert_eq!(stage.features[0].feature_id, feature_id);

        // Works with a partially filled stage.
        let staged_features = vec![Pubkey::new_unique(); 4];
        let mut stage = setup_stage(&staged_features);
        assert_eq!(stage.stage(&feature_id), Ok(()));
        assert_eq!(stage.features[4].feature_id, feature_id);

        // Works with an almost full stage.
        let staged_features = vec![Pubkey::new_unique(); MAX_FEATURES - 1];
        let mut stage = setup_stage(&staged_features);
        assert_eq!(stage.stage(&feature_id), Ok(()));
        assert_eq!(stage.features[MAX_FEATURES - 1].feature_id, feature_id);
    }

    fn test_to_from_bitmask(val: u8, signal: [bool; 8]) {
        let mask = FeatureBitMask(val);
        assert_eq!(mask.0, val);
        assert_eq!(<[bool; 8]>::from(&mask), signal);
        assert_eq!(FeatureBitMask::from(&signal), mask);
    }

    #[test]
    fn test_bitmask() {
        test_to_from_bitmask(
            0b00000000,
            [false, false, false, false, false, false, false, false],
        );
        test_to_from_bitmask(
            0b00000001,
            [false, false, false, false, false, false, false, true],
        );
        test_to_from_bitmask(
            0b00000010,
            [false, false, false, false, false, false, true, false],
        );
        test_to_from_bitmask(
            0b00000011,
            [false, false, false, false, false, false, true, true],
        );
        test_to_from_bitmask(
            0b01010101,
            [false, true, false, true, false, true, false, true],
        );
        test_to_from_bitmask(
            0b10001101,
            [true, false, false, false, true, true, false, true],
        );
        test_to_from_bitmask(0b11111111, [true, true, true, true, true, true, true, true]);
    }

    #[test]
    fn test_add_stake_support() {
        let features = vec![Pubkey::new_unique(); MAX_FEATURES];
        let mut stage = setup_stage(&features);

        stage.add_stake_support(&FeatureBitMask(0b00000001), 0, 100_000_000);
        assert_eq!(stage.features[7].stake_support, 100_000_000);

        stage.add_stake_support(&FeatureBitMask(0b00000001), 0, 100_000_000);
        assert_eq!(stage.features[7].stake_support, 200_000_000);

        stage.add_stake_support(&FeatureBitMask(0b01010101), 0, 100_000_000);
        assert_eq!(stage.features[1].stake_support, 100_000_000);
        assert_eq!(stage.features[3].stake_support, 100_000_000);
        assert_eq!(stage.features[5].stake_support, 100_000_000);
        assert_eq!(stage.features[7].stake_support, 300_000_000);

        stage.add_stake_support(&FeatureBitMask(0b01010100), 0, 100_000_000);
        assert_eq!(stage.features[1].stake_support, 200_000_000);
        assert_eq!(stage.features[3].stake_support, 200_000_000);
        assert_eq!(stage.features[5].stake_support, 200_000_000);
        assert_eq!(stage.features[7].stake_support, 300_000_000);
    }
}
