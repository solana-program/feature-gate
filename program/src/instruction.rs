//! Program instructions

use {
    crate::state::FeatureBitMask,
    shank::ShankInstruction,
    solana_program::{
        incinerator,
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

/// Feature Gate program instructions
#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, ShankInstruction)]
#[repr(u8)]
pub enum FeatureGateInstruction {
    /// Revoke a pending feature activation.
    ///
    /// This instruction will burn any lamports in the feature account.
    ///
    /// A "pending" feature activation is a feature account that has been
    /// allocated and assigned, but hasn't yet been updated by the runtime
    /// with an `activation_slot`.
    ///
    /// Features that _have_ been activated by the runtime cannot be revoked.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[w+s]`    Feature account
    ///   1. `[w]`      Incinerator
    ///   2. `[ ]`      System program
    #[account(
        0,
        writable,
        signer,
        name = "feature",
        description = "The feature account to revoke"
    )]
    #[account(
        1,
        writable,
        name = "incinerator",
        description = "The incinerator account"
    )]
    #[account(
        2,
        name = "system_program",
        description = "The system program"
    )]
    RevokePendingActivation,
    /// Add a feature to the set of features staged for activation at the end of
    /// the next epoch.
    ///
    /// Features submitted to this instruction during epoch N-1 will be staged
    /// for activation at the end of epoch N. This instruction can only be
    /// invoked by the designated staging authority.
    ///
    /// This instruction expects the staged features account to either exist or
    /// have been allocated enough space and owned by the Feature Gate program,
    /// in order to initialize state. If the account is not yet initialized, it
    /// will be initialized before the new feature is added.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[ ]`  Feature account
    ///   1. `[w]`  Staged features account
    ///   2. `[s]`  Staging authority
    #[account(
        0,
        name = "feature",
        description = "The feature account to stage"
    )]
    #[account(
        1,
        writable,
        name = "staged_features",
        description = "The staged features account"
    )]
    #[account(
        2,
        signer,
        name = "staging_authority",
        description = "The staging authority"
    )]
    StageFeatureForActivation,
    /// Signal stake support for staged features.
    ///
    /// This instruction will lookup the provided vote account's total stake
    /// for the current epoch, then interpret the provided bitmask, then use
    /// their stake amount to update the stake in support for the features in
    /// the staged features account.
    /// 
    /// This instruction expects the validator support signal account to either
    /// exist or have been allocated enough space and owned by the Feature Gate
    /// program, in order to initialize state. If the account is not yet
    /// initialized, it will be initialized, and the validator's stake will be
    /// added to each supported feature.
    /// 
    /// If the validator support signal account already contains the validator's
    /// last submitted bitmask for the current epoch, the validator's previous
    /// bitmask will be used to deduct stake support before adding the new stake
    /// support, to account for changes in the validator's support signal.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[w]`      Staged features account
    ///   1. `[w]`      Validator support signal account
    ///   2. `[ ]`      Vote account
    ///   3. `[s]`      Authorized voter account
    #[account(
        0,
        writable,
        name = "staged_features",
        description = "The staged features account"
    )]
    #[account(
        1,
        writable,
        name = "validator_support_signal",
        description = "The validator support signal account"
    )]
    #[account(
        2,
        name = "vote_account",
        description = "The vote account"
    )]
    #[account(
        3,
        signer,
        name = "authorized_voter",
        description = "authorized voter account"
    )]
    SignalSupportForStagedFeatures {
        /// The bitmask of features supported.
        signal: FeatureBitMask,
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
                let signal = FeatureBitMask(rest[0]);
                Ok(Self::SignalSupportForStagedFeatures { signal })
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
            Self::SignalSupportForStagedFeatures { signal } => vec![2, signal.into()],
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
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*feature_id, false),
        AccountMeta::new(*staged_features_address, false),
        AccountMeta::new_readonly(*staging_authority, true),
    ];

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
    validator_support_signal_address: &Pubkey,
    vote_account_address: &Pubkey,
    authorized_voter_address: &Pubkey,
    signal: FeatureBitMask,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*staged_features_address, false),
        AccountMeta::new(*validator_support_signal_address, false),
        AccountMeta::new_readonly(*vote_account_address, false),
        AccountMeta::new_readonly(*authorized_voter_address, true),
    ];

    let data = FeatureGateInstruction::SignalSupportForStagedFeatures { signal }.pack();

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
            signal: FeatureBitMask(0),
        });
        test_pack_unpack(&FeatureGateInstruction::SignalSupportForStagedFeatures {
            signal: FeatureBitMask(1),
        });
        test_pack_unpack(&FeatureGateInstruction::SignalSupportForStagedFeatures {
            signal: FeatureBitMask(255),
        });
    }
}
