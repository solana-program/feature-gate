//! Program state types.
use {
    crate::error::FeatureGateError,
    bytemuck::{Pod, Zeroable},
    solana_program::{clock::Epoch, program_error::ProgramError, pubkey::Pubkey},
};

/// The maximum number of features that can be staged per epoch.
pub const MAX_FEATURES: usize = 8;
/// The maxmimum number of historical support signals stored in a Validator
/// Support Signal account.
pub const MAX_SIGNALS: usize = 4;

/// The seed prefix (`"staged_features"`) in bytes used to derive the address
/// of a staged features account.
/// Seeds: `"staged_features" + epoch`, where epoch is serialized to eight
/// little-endian bytes.
pub const SEED_PREFIX_STAGED_FEATURES: &[u8] = b"staged_features";
/// The seed prefix (`"support_signal"`) in bytes used to derive the address
/// of a validator support signal account.
/// Seeds: `"support_signal" + vote_address`.
pub const SEED_PREFIX_SUPPORT_SIGNAL: &[u8] = b"support_signal";

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

/// Derive the address of a validator support signal account.
pub fn get_validator_support_signal_address(vote_account_address: &Pubkey) -> Pubkey {
    get_validator_support_signal_address_and_bump_seed(vote_account_address).0
}

/// Derive the address of a validator support signal account, with bump seed.
pub fn get_validator_support_signal_address_and_bump_seed(
    vote_account_address: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &collect_validator_support_signal_seeds(vote_account_address),
        &crate::id(),
    )
}

pub(crate) fn collect_validator_support_signal_seeds(vote_account_address: &Pubkey) -> [&[u8]; 2] {
    [SEED_PREFIX_SUPPORT_SIGNAL, vote_account_address.as_ref()]
}

/// A Feature ID and its corresponding stake support, as signalled by
/// validators.
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
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
}

/// Features staged for activation at the end of the epoch, with their
/// corresponding signalled stake support.
///
/// Supports a maximum of `MAX_FEATURES` features for any given epoch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
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
}

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
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
#[repr(transparent)]
pub struct FeatureBitMask(pub u8);

impl From<&FeatureBitMask> for u8 {
    fn from(mask: &FeatureBitMask) -> u8 {
        mask.0
    }
}

/// A validator's support signal bitmask along with the epoch the signal
/// corresponds to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct SupportSignalWithEpoch {
    /// The epoch the support signal corresponds to.
    pub epoch: Epoch,
    /// The support signal bitmask.
    pub signal: FeatureBitMask,
    _padding: [u8; 7],
}

/// A validator's support signal bitmasks with their corresponding epochs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct ValidatorSupportSignal {
    /// The support signal bitmasks with their corresponding epochs.
    pub signals: [SupportSignalWithEpoch; MAX_SIGNALS],
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_stage(feature_ids: &[Pubkey]) -> StagedFeatures {
        let mut stage = StagedFeatures::default();
        for (i, id) in feature_ids.iter().enumerate() {
            stage.features[i].feature_id = *id;
        }
        stage
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
}
