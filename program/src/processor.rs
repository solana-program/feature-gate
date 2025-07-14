//! Program state processor

use {
    crate::{error::FeatureGateError, instruction::FeatureGateInstruction},
    solana_account_info::{next_account_info, AccountInfo},
    solana_cpi::invoke,
    solana_feature_gate_interface::Feature,
    solana_msg::msg,
    solana_program_error::{ProgramError, ProgramResult},
    solana_pubkey::Pubkey,
    solana_sdk_ids::incinerator,
    solana_system_interface::{instruction as system_instruction, program as system_program},
};

/// Processes a [RevokePendingActivation](enum.FeatureGateInstruction.html)
/// instruction.
pub fn process_revoke_pending_activation(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let feature_info = next_account_info(account_info_iter)?;
    let incinerator_info = next_account_info(account_info_iter)?;
    let _system_program_info = next_account_info(account_info_iter)?;

    if !feature_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // This will also check the program ID
    if Feature::from_account_info(feature_info)?
        .activated_at
        .is_some()
    {
        return Err(FeatureGateError::FeatureAlreadyActivated.into());
    }

    // Clear data and reassign.
    feature_info.resize(0)?;
    feature_info.assign(&system_program::id());

    // Burn the lamports.
    let lamports = feature_info.lamports();
    invoke(
        &system_instruction::transfer(feature_info.key, &incinerator::id(), lamports),
        &[feature_info.clone(), incinerator_info.clone()],
    )?;

    Ok(())
}

/// Processes an [Instruction](enum.Instruction.html).
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = FeatureGateInstruction::unpack(input)?;
    match instruction {
        FeatureGateInstruction::RevokePendingActivation => {
            msg!("Instruction: RevokePendingActivation");
            process_revoke_pending_activation(program_id, accounts)
        }
    }
}
