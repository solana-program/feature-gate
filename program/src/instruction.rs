//! Program instructions

use {
    num_enum::{IntoPrimitive, TryFromPrimitive},
    shank::ShankInstruction,
    solana_instruction::{AccountMeta, Instruction},
    solana_program_error::ProgramError,
    solana_pubkey::Pubkey,
    solana_sdk_ids::incinerator,
    solana_system_interface::program as system_program,
};

/// Feature Gate program instructions
#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, IntoPrimitive, ShankInstruction, TryFromPrimitive)]
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
}
impl FeatureGateInstruction {
    /// Unpacks a byte buffer into a
    /// [`FeatureGateInstruction`](enum.FeatureGateInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() != 1 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Self::try_from(input[0]).map_err(|_| ProgramError::InvalidInstructionData)
    }

    /// Packs a [`FeatureGateInstruction`](enum.FeatureGateInstruction.html)
    /// into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        vec![self.to_owned().into()]
    }
}

/// Creates a `RevokePendingActivation` instruction.
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
}
