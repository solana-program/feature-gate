//! Program state types.
use {
    crate::error::FeatureGateError,
    bytemuck::{Pod, Zeroable},
    solana_program::{clock::Epoch, program_error::ProgramError, pubkey::Pubkey},
};

/// The maximum number of features that can be staged per epoch.
pub const MAX_FEATURES: usize = 8;
/// The maxmimum number of historical support signals stored in a Validator
/// Support Signal account. If stored sequentially, signals are rotated out
/// after slightly over a week (4 epochs).
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

    fn add_stake_support(&mut self, stake: u64) {
        self.stake_support = self.stake_support.saturating_add(stake);
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
    /// Add stake support for features given a bitmask signal.
    pub fn add_stake_support(&mut self, signal: &FeatureBitMask, vote_account_epoch_stake: u64) {
        self.features
            .iter_mut()
            .zip(<[bool; MAX_FEATURES]>::from(signal).iter())
            .for_each(|(feature, signal)| {
                if *signal && feature.is_initialized() {
                    feature.add_stake_support(vote_account_epoch_stake);
                }
            });
    }

    /// Deduct stake support for features given a bitmask signal.
    pub fn deduct_stake_support(&mut self, signal: &FeatureBitMask, vote_account_epoch_stake: u64) {
        self.features
            .iter_mut()
            .zip(<[bool; MAX_FEATURES]>::from(signal).iter())
            .for_each(|(feature, signal)| {
                if *signal && feature.is_initialized() {
                    feature.stake_support = feature
                        .stake_support
                        .saturating_sub(vote_account_epoch_stake);
                }
            });
    }

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

impl From<&FeatureBitMask> for [bool; MAX_FEATURES] {
    fn from(mask: &FeatureBitMask) -> [bool; MAX_FEATURES] {
        let mut signals = [false; MAX_FEATURES];
        for i in 0..MAX_FEATURES {
            if let Some(idx) = MAX_FEATURES.checked_sub(1).and_then(|d| d.checked_sub(i)) {
                signals[idx] = mask.0 & (1 << i) != 0;
            }
        }
        signals
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

impl SupportSignalWithEpoch {
    fn new(epoch: Epoch, signal: FeatureBitMask) -> Self {
        Self {
            epoch,
            signal,
            _padding: [0; 7],
        }
    }
}

/// A validator's support signal bitmasks with their corresponding epochs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct ValidatorSupportSignal {
    /// The support signal bitmasks with their corresponding epochs.
    pub signals: [SupportSignalWithEpoch; MAX_SIGNALS],
}

impl ValidatorSupportSignal {
    /// Get a stored signal for a given epoch.
    pub fn get_signal_for_epoch(&self, epoch: Epoch) -> Option<&FeatureBitMask> {
        self.signals
            .iter()
            .find(|signal| signal.epoch == epoch)
            .map(|signal| &signal.signal)
    }

    /// Add a support signal for a given epoch. If the signal already exists,
    /// it is updated. If the list is full, the last (oldest) signal is dropped
    /// and the new signal is added.
    pub fn store_signal(&mut self, epoch: Epoch, signal: FeatureBitMask) {
        if let Some(existing) = self.signals.iter_mut().find(|signal| signal.epoch == epoch) {
            existing.signal = signal;
            return;
        }
        self.signals.rotate_right(1);
        self.signals[0] = SupportSignalWithEpoch::new(epoch, signal);
    }
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

    fn test_unpack_bitmask(val: u8, unpacked: &[bool; 8]) {
        let mask = FeatureBitMask(val);
        assert_eq!(mask.0, val);
        assert_eq!(&<[bool; MAX_FEATURES]>::from(&mask), unpacked);
    }

    #[test]
    fn test_bitmask() {
        test_unpack_bitmask(
            0b00000000,
            &[false, false, false, false, false, false, false, false],
        );
        test_unpack_bitmask(
            0b00000001,
            &[false, false, false, false, false, false, false, true],
        );
        test_unpack_bitmask(
            0b00000010,
            &[false, false, false, false, false, false, true, false],
        );
        test_unpack_bitmask(
            0b00000011,
            &[false, false, false, false, false, false, true, true],
        );
        test_unpack_bitmask(
            0b01010101,
            &[false, true, false, true, false, true, false, true],
        );
        test_unpack_bitmask(
            0b10001101,
            &[true, false, false, false, true, true, false, true],
        );
        test_unpack_bitmask(
            0b11111111,
            &[true, true, true, true, true, true, true, true],
        );
    }

    #[test]
    fn test_add_stake_support() {
        let mut features = Vec::with_capacity(MAX_FEATURES);
        for _ in 0..MAX_FEATURES {
            features.push(Pubkey::new_unique());
        }
        let mut stage = setup_stage(&features);

        // Signal support for feature 8 (index 7).
        stage.add_stake_support(&FeatureBitMask(0b00000001), 100_000_000);
        assert_eq!(stage.features[7].stake_support, 100_000_000);

        // Signal support for feature 8 again.
        stage.add_stake_support(&FeatureBitMask(0b00000001), 100_000_000);
        assert_eq!(stage.features[7].stake_support, 200_000_000);

        // Signal support for features 2, 4, 6, 8 (indices 1, 3, 5, 7).
        stage.add_stake_support(&FeatureBitMask(0b01010101), 100_000_000);
        assert_eq!(stage.features[0].stake_support, 0);
        assert_eq!(stage.features[1].stake_support, 100_000_000);
        assert_eq!(stage.features[2].stake_support, 0);
        assert_eq!(stage.features[3].stake_support, 100_000_000);
        assert_eq!(stage.features[4].stake_support, 0);
        assert_eq!(stage.features[5].stake_support, 100_000_000);
        assert_eq!(stage.features[6].stake_support, 0);
        assert_eq!(stage.features[7].stake_support, 300_000_000);

        // Signal support for features 2, 4, 6, 8 again.
        stage.add_stake_support(&FeatureBitMask(0b01010101), 100_000_000);
        assert_eq!(stage.features[0].stake_support, 0);
        assert_eq!(stage.features[1].stake_support, 200_000_000);
        assert_eq!(stage.features[2].stake_support, 0);
        assert_eq!(stage.features[3].stake_support, 200_000_000);
        assert_eq!(stage.features[4].stake_support, 0);
        assert_eq!(stage.features[5].stake_support, 200_000_000);
        assert_eq!(stage.features[6].stake_support, 0);
        assert_eq!(stage.features[7].stake_support, 400_000_000);

        // Signal support for features 3, 5, 7 (indices 2, 4, 6).
        stage.add_stake_support(&FeatureBitMask(0b00101010), 100_000_000);
        assert_eq!(stage.features[0].stake_support, 0);
        assert_eq!(stage.features[1].stake_support, 200_000_000);
        assert_eq!(stage.features[2].stake_support, 100_000_000);
        assert_eq!(stage.features[3].stake_support, 200_000_000);
        assert_eq!(stage.features[4].stake_support, 100_000_000);
        assert_eq!(stage.features[5].stake_support, 200_000_000);
        assert_eq!(stage.features[6].stake_support, 100_000_000);
        assert_eq!(stage.features[7].stake_support, 400_000_000);
    }

    #[test]
    fn test_deduct_stake_support() {
        let mut stage = setup_stage(&[]);
        for f in stage.features.iter_mut() {
            f.feature_id = Pubkey::new_unique();
            f.stake_support = 400_000_000;
        }

        // Deduct support for feature 8 (index 7).
        stage.deduct_stake_support(&FeatureBitMask(0b00000001), 100_000_000);
        assert_eq!(stage.features[7].stake_support, 300_000_000);

        // Deduct support for feature 8 again.
        stage.deduct_stake_support(&FeatureBitMask(0b00000001), 100_000_000);
        assert_eq!(stage.features[7].stake_support, 200_000_000);

        // Deduct support for features 2, 4, 6, 8 (indices 1, 3, 5, 7).
        stage.deduct_stake_support(&FeatureBitMask(0b01010101), 100_000_000);
        assert_eq!(stage.features[0].stake_support, 400_000_000);
        assert_eq!(stage.features[1].stake_support, 300_000_000);
        assert_eq!(stage.features[2].stake_support, 400_000_000);
        assert_eq!(stage.features[3].stake_support, 300_000_000);
        assert_eq!(stage.features[4].stake_support, 400_000_000);
        assert_eq!(stage.features[5].stake_support, 300_000_000);
        assert_eq!(stage.features[6].stake_support, 400_000_000);
        assert_eq!(stage.features[7].stake_support, 100_000_000);

        // Deduct support for features 2, 4, 6, 8 again.
        stage.deduct_stake_support(&FeatureBitMask(0b01010101), 100_000_000);
        assert_eq!(stage.features[0].stake_support, 400_000_000);
        assert_eq!(stage.features[1].stake_support, 200_000_000);
        assert_eq!(stage.features[2].stake_support, 400_000_000);
        assert_eq!(stage.features[3].stake_support, 200_000_000);
        assert_eq!(stage.features[4].stake_support, 400_000_000);
        assert_eq!(stage.features[5].stake_support, 200_000_000);
        assert_eq!(stage.features[6].stake_support, 400_000_000);
        assert_eq!(stage.features[7].stake_support, 0);

        // Deduct support for features 3, 5, 7 (indices 2, 4, 6).
        stage.deduct_stake_support(&FeatureBitMask(0b00101010), 100_000_000);
        assert_eq!(stage.features[0].stake_support, 400_000_000);
        assert_eq!(stage.features[1].stake_support, 200_000_000);
        assert_eq!(stage.features[2].stake_support, 300_000_000);
        assert_eq!(stage.features[3].stake_support, 200_000_000);
        assert_eq!(stage.features[4].stake_support, 300_000_000);
        assert_eq!(stage.features[5].stake_support, 200_000_000);
        assert_eq!(stage.features[6].stake_support, 300_000_000);
        assert_eq!(stage.features[7].stake_support, 0);
    }

    #[test]
    fn test_store_signal() {
        let mut signals = ValidatorSupportSignal::default();

        // Store a signal for epoch 0.
        signals.store_signal(0, FeatureBitMask(0b00000001));
        assert_eq!(signals.signals[0].epoch, 0);
        assert_eq!(signals.signals[0].signal.0, 0b00000001);

        // Store a signal for epoch 1.
        signals.store_signal(1, FeatureBitMask(0b00000010));
        assert_eq!(signals.signals[0].epoch, 1);
        assert_eq!(signals.signals[0].signal.0, 0b00000010);
        assert_eq!(signals.signals[1].epoch, 0);
        assert_eq!(signals.signals[1].signal.0, 0b00000001);

        // Store a signal for epoch 2.
        signals.store_signal(2, FeatureBitMask(0b00000100));
        assert_eq!(signals.signals[0].epoch, 2);
        assert_eq!(signals.signals[0].signal.0, 0b00000100);
        assert_eq!(signals.signals[1].epoch, 1);
        assert_eq!(signals.signals[1].signal.0, 0b00000010);
        assert_eq!(signals.signals[2].epoch, 0);
        assert_eq!(signals.signals[2].signal.0, 0b00000001);

        // Store a new signal for epoch 2.
        // It should update the existing entry.
        signals.store_signal(2, FeatureBitMask(0b11000000));
        assert_eq!(signals.signals[0].epoch, 2);
        assert_eq!(signals.signals[0].signal.0, 0b11000000);
        assert_eq!(signals.signals[1].epoch, 1);
        assert_eq!(signals.signals[1].signal.0, 0b00000010);
        assert_eq!(signals.signals[2].epoch, 0);
        assert_eq!(signals.signals[2].signal.0, 0b00000001);

        // Store a signal for epoch 3.
        signals.store_signal(3, FeatureBitMask(0b00001000));
        assert_eq!(signals.signals[0].epoch, 3);
        assert_eq!(signals.signals[0].signal.0, 0b00001000);
        assert_eq!(signals.signals[1].epoch, 2);
        assert_eq!(signals.signals[1].signal.0, 0b11000000);
        assert_eq!(signals.signals[2].epoch, 1);
        assert_eq!(signals.signals[2].signal.0, 0b00000010);
        assert_eq!(signals.signals[3].epoch, 0);
        assert_eq!(signals.signals[3].signal.0, 0b00000001);

        // Store a signal for epoch 4.
        // Epoch 0 should be rotated out.
        signals.store_signal(4, FeatureBitMask(0b00010000));
        assert_eq!(signals.signals[0].epoch, 4);
        assert_eq!(signals.signals[0].signal.0, 0b00010000);
        assert_eq!(signals.signals[1].epoch, 3);
        assert_eq!(signals.signals[1].signal.0, 0b00001000);
        assert_eq!(signals.signals[2].epoch, 2);
        assert_eq!(signals.signals[2].signal.0, 0b11000000);
        assert_eq!(signals.signals[3].epoch, 1);
        assert_eq!(signals.signals[3].signal.0, 0b00000010);
    }
}
