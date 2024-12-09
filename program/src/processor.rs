//! Program state processor

use {
    crate::{
        error::FeatureGateError,
        instruction::FeatureGateInstruction,
        state::{
            get_staged_features_address, get_validator_support_signal_address, FeatureBitMask,
            StagedFeatures, ValidatorSupportSignal,
        },
    },
    bytemuck::Pod,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::Clock,
        entrypoint::ProgramResult,
        epoch_stake::get_epoch_stake_for_vote_account,
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

fn unpack_pod_mut<P: Pod>(data: &mut [u8]) -> Result<&mut P, ProgramError> {
    bytemuck::try_from_bytes_mut(data).map_err(|_| ProgramError::InvalidAccountData)
}

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
    unpack_pod_mut::<StagedFeatures>(&mut staged_features_data)
        .and_then(|s| s.stage(feature_info.key))?;

    Ok(())
}

/// Processes a
/// [SignalSupportForStagedFeatures](enum.FeatureGateInstruction.html)
/// instruction.
fn process_signal_support_for_staged_features(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    signal: FeatureBitMask,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let staged_features_info = next_account_info(account_info_iter)?;
    let validator_support_signal_info = next_account_info(account_info_iter)?;
    let vote_account_info = next_account_info(account_info_iter)?;
    let authorized_voter_info = next_account_info(account_info_iter)?;

    // Ensure the authorized voter account is a signer.
    if !authorized_voter_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load the clock sysvar to get the _current_ epoch.
    let clock = <Clock as Sysvar>::get()?;
    let current_epoch = clock.epoch;

    // Ensure the provided authorized voter is the correct authorized voter for
    // the provided vote account.
    // Also validates vote account state.
    {
        if !vote_account_info
            .owner
            .eq(&solana_program::vote::program::id())
        {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Authorized voter pubkey is at offset 36. Don't deserialize the whole
        // vote account.
        let vote_data = vote_account_info.try_borrow_data()?;
        let vote_state_authorized_voter = vote_data
            .get(36..68)
            .and_then(|slice| slice.try_into().ok())
            .map(Pubkey::new_from_array)
            .ok_or(ProgramError::InvalidAccountData)?;

        if !vote_state_authorized_voter.eq(authorized_voter_info.key) {
            return Err(ProgramError::IncorrectAuthority);
        }
    }

    // Ensure the staged features address is the correct address derived
    // from the _current_ epoch.
    if !staged_features_info
        .key
        .eq(&get_staged_features_address(&current_epoch))
    {
        return Err(FeatureGateError::IncorrectStagedFeaturesAddress.into());
    }

    // Ensure the staged features account is owned by the program.
    if staged_features_info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut staged_features_data = staged_features_info.try_borrow_mut_data()?;
    let staged_features = unpack_pod_mut::<StagedFeatures>(&mut staged_features_data)?;

    // Get the provided vote account's epoch stake.
    let vote_account_epoch_stake = get_epoch_stake_for_vote_account(vote_account_info.key);

    if vote_account_epoch_stake != 0 {
        // Ensure the validator support signal address is the correct address
        // derived from the vote account and the _current_ epoch.
        if !validator_support_signal_info
            .key
            .eq(&get_validator_support_signal_address(vote_account_info.key))
        {
            return Err(FeatureGateError::IncorrectValidatorSupportSignalAddress.into());
        }

        // Load the validator's last signal.
        let mut validator_support_signal_data =
            validator_support_signal_info.try_borrow_mut_data()?;
        let validator_support_signal_state =
            unpack_pod_mut::<ValidatorSupportSignal>(&mut validator_support_signal_data)?;

        // First deduct from the staged features the stake the validator
        // previously signaled support for.
        if let Some(last_signal) =
            validator_support_signal_state.get_signal_for_epoch(current_epoch)
        {
            staged_features.deduct_stake_support(last_signal, vote_account_epoch_stake);
        }

        // Add the validator's stake in support of the _new_ signaled
        // features.
        staged_features.add_stake_support(&signal, vote_account_epoch_stake);

        // Overwrite the validator's last signal.
        validator_support_signal_state.store_signal(current_epoch, signal);
    }

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
