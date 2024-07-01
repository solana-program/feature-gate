//! Program state processor

use {
    crate::{error::FeatureGateError, instruction::FeatureGateInstruction, state::FeatureBitMask},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        feature::Feature,
        incinerator, msg,
        program::invoke,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction, system_program,
    },
};

/// Processes a [RevokePendingActivation](enum.FeatureGateInstruction.html)
/// instruction.
fn process_revoke_pending_activation(
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
    feature_info.realloc(0, true)?;
    feature_info.assign(&system_program::id());

    // Burn the lamports.
    let lamports = feature_info.lamports();
    invoke(
        &system_instruction::transfer(feature_info.key, &incinerator::id(), lamports),
        &[feature_info.clone(), incinerator_info.clone()],
    )?;

    Ok(())
}

/// Processes a [StageFeatureForActivation](enum.FeatureGateInstruction.html)
/// instruction.
fn process_stage_feature_for_activation(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
) -> ProgramResult {
    Ok(())
}

/// Processes a
/// [SignalSupportForStagedFeatures](enum.FeatureGateInstruction.html)
/// instruction.
fn process_signal_support_for_staged_features(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _features: FeatureBitMask,
) -> ProgramResult {
    Ok(())
}

/// Processes a [FeatureGateInstruction](enum.FeatureGateInstruction.html).
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = FeatureGateInstruction::unpack(input)?;
    match instruction {
        FeatureGateInstruction::RevokePendingActivation => {
            msg!("Instruction: RevokePendingActivation");
            process_revoke_pending_activation(program_id, accounts)
        }
        FeatureGateInstruction::StageFeatureForActivation => {
            msg!("Instruction: StageFeatureForActivation");
            process_stage_feature_for_activation(program_id, accounts)
        }
        FeatureGateInstruction::SignalSupportForStagedFeatures { features } => {
            msg!("Instruction: SignalSupportForStagedFeatures");
            process_signal_support_for_staged_features(program_id, accounts, features)
        }
    }
}
