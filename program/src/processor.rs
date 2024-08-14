//! Program state processor

use {
    crate::{
        error::FeatureGateError,
        instruction::FeatureGateInstruction,
        state::{get_staged_features_address, FeatureBitMask, StagedFeatures},
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::Clock,
        entrypoint::ProgramResult,
        feature::Feature,
        incinerator, msg,
        program::invoke,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction, system_program,
        sysvar::Sysvar,
    },
};

// Simply the public key of a keypair created with seed `[0; 32]`, for now.
const STAGE_AUTHORITY_ADDRESS: Pubkey = Pubkey::new_from_array([
    59, 106, 39, 188, 206, 182, 164, 45, 98, 163, 168, 208, 42, 111, 13, 115, 101, 50, 21, 119, 29,
    226, 67, 166, 58, 192, 72, 161, 139, 89, 218, 41,
]);

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
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let feature_info = next_account_info(account_info_iter)?;
    let staged_features_info = next_account_info(account_info_iter)?;
    let stage_authority_info = next_account_info(account_info_iter)?;

    // Ensure the proper authority was provided as a signer.
    if !stage_authority_info.key.eq(&STAGE_AUTHORITY_ADDRESS) {
        return Err(ProgramError::IncorrectAuthority);
    }
    if !stage_authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ensure the feature exists and has not been activated.
    // This will also check the owner program ID.
    if Feature::from_account_info(feature_info)?
        .activated_at
        .is_some()
    {
        return Err(FeatureGateError::FeatureAlreadyActivated.into());
    }

    // Load the clock sysvar to get the current epoch.
    let clock = <Clock as Sysvar>::get()?;
    let upcoming_epoch = clock.epoch.saturating_add(1);

    // Ensure the staged features address is the correct address derived
    // from the upcoming epoch.

    if !staged_features_info
        .key
        .eq(&get_staged_features_address(&upcoming_epoch))
    {
        return Err(FeatureGateError::IncorrectStagedFeaturesAddress.into());
    }

    // Stage the feature. The `stage` method will ensure the feature is not
    // already staged.
    let mut staged_features_data = staged_features_info.try_borrow_mut_data()?;
    bytemuck::try_from_bytes_mut::<StagedFeatures>(&mut staged_features_data)
        .map_err(|_| ProgramError::InvalidAccountData)
        .and_then(|staged_features| staged_features.stage(feature_info.key))?;

    Ok(())
}

/// Processes a
/// [SignalSupportForStagedFeatures](enum.FeatureGateInstruction.html)
/// instruction.
fn process_signal_support_for_staged_features(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _signal: FeatureBitMask,
) -> ProgramResult {
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
        FeatureGateInstruction::StageFeatureForActivation => {
            msg!("Instruction: StageFeatureForActivation");
            process_stage_feature_for_activation(program_id, accounts)
        }
        FeatureGateInstruction::SignalSupportForStagedFeatures { signal } => {
            msg!("Instruction: SignalSupportForStagedFeatures");
            process_signal_support_for_staged_features(program_id, accounts, signal)
        }
    }
}
