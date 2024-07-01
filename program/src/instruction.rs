//! Program instructions

use {
    crate::state::FeatureBitMask,
    solana_program::{
        incinerator,
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

/// Feature Gate program instructions
#[derive(Clone, Debug, PartialEq)]
pub enum FeatureGateInstruction {
    /// Revoke a pending feature activation.
    ///
    /// This instruction will burn any lamports in the feature account.
    ///
    /// A "pending" feature activation is a feature account that is included
    /// in the current epoch's staged features account, and is scheduled to be
    /// activated at the end of the current epoch, pending stake support.
    ///
    /// Features that _have_ been activated by the runtime cannot be revoked.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[w+s]`    Feature account
    ///   1. `[w]`      Incinerator
    ///   2. `[ ]`      System program
    RevokePendingActivation,
    /// Add a feature to the set of features staged for activation at the end of
    /// the next epoch.
    ///
    /// Features submitted to this instruction during epoch N-1 will be staged
    /// for activation at the end of epoch N. This instruction can only be
    /// invoked by the designated staging authority.
    ///
    /// This instruction expects the staged features account to either exist or
    /// contain enough lamports to be rent-exempt after initialization. If the
    /// account is not yet initialized, it will be initialized before the new
    /// feature is added.
    ///
    /// If the account must be initialized, the system program must be
    /// provided.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[ ]`      Feature account
    ///   1. `[w]`      Staged features account
    ///   2. `[s]`      Staging authority
    ///   3. `[op: ]`   System program
    StageFeatureForActivation,
    /// Signal stake support for staged features.
    ///
    /// This instruction will lookup the provided vote account's total stake
    /// for the current epoch, then interpret the provided bitmask, then add
    /// their stake amount to each supported feature in the staged features
    /// account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[w]`      Staged features account
    ///   1. `[s]`      Vote account
    SignalSupportForStagedFeatures {
        /// The bitmask of features supported.
        features: FeatureBitMask,
    },
}
impl FeatureGateInstruction {
    /// Unpacks a byte buffer into a
    /// [FeatureGateInstruction](enum.FeatureGateInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input.split_first() {
            Some((&0, _)) => Ok(Self::RevokePendingActivation),
            Some((&1, _)) => Ok(Self::StageFeatureForActivation),
            Some((&2, rest)) if rest.len() == 1 => {
                let features = FeatureBitMask(rest[0]);
                Ok(Self::SignalSupportForStagedFeatures { features })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }

    /// Packs a [FeatureGateInstruction](enum.FeatureGateInstruction.html) into
    /// a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        match self {
            Self::RevokePendingActivation => vec![0],
            Self::StageFeatureForActivation => vec![1],
            Self::SignalSupportForStagedFeatures { features } => vec![2, features.0],
        }
    }
}

/// Creates a [RevokePendingActivation](enum.FeatureGateInstruction.html)
/// instruction.
pub fn revoke_pending_activation(feature_id: &Pubkey) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*feature_id, true),
        AccountMeta::new(incinerator::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    let data = FeatureGateInstruction::RevokePendingActivation.pack();

    Instruction {
        program_id: crate::id(),
        accounts,
        data,
    }
}

/// Creates a [StageFeatureForActivation](enum.FeatureGateInstruction.html)
/// instruction.
pub fn stage_feature_for_activation(
    feature_id: &Pubkey,
    staged_features_address: &Pubkey,
    staging_authority: &Pubkey,
    init: bool,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new_readonly(*feature_id, false),
        AccountMeta::new(*staged_features_address, false),
        AccountMeta::new_readonly(*staging_authority, true),
    ];

    if init {
        accounts.push(AccountMeta::new_readonly(system_program::id(), false));
    }

    let data = FeatureGateInstruction::StageFeatureForActivation.pack();

    Instruction {
        program_id: crate::id(),
        accounts,
        data,
    }
}

/// Creates a [SignalSupportForStagedFeatures](enum.FeatureGateInstruction.html)
/// instruction.
pub fn signal_support_for_staged_features(
    staged_features_address: &Pubkey,
    vote_account_address: &Pubkey,
    features: FeatureBitMask,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*staged_features_address, false),
        AccountMeta::new_readonly(*vote_account_address, true),
    ];

    let data = FeatureGateInstruction::SignalSupportForStagedFeatures { features }.pack();

    Instruction {
        program_id: crate::id(),
        accounts,
        data,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_pack_unpack(instruction: &FeatureGateInstruction) {
        let packed = instruction.pack();
        let unpacked = FeatureGateInstruction::unpack(&packed).unwrap();
        assert_eq!(instruction, &unpacked);
    }

    #[test]
    fn test_pack_unpack_revoke_pending_activation() {
        test_pack_unpack(&FeatureGateInstruction::RevokePendingActivation);
    }

    #[test]
    fn test_pack_unpack_stage_feature_for_activation() {
        test_pack_unpack(&FeatureGateInstruction::StageFeatureForActivation);
    }

    #[test]
    fn test_pack_unpack_signal_support_for_staged_features() {
        test_pack_unpack(&FeatureGateInstruction::SignalSupportForStagedFeatures {
            features: FeatureBitMask(0),
        });
        test_pack_unpack(&FeatureGateInstruction::SignalSupportForStagedFeatures {
            features: FeatureBitMask(1),
        });
        test_pack_unpack(&FeatureGateInstruction::SignalSupportForStagedFeatures {
            features: FeatureBitMask(255),
        });
    }
}
