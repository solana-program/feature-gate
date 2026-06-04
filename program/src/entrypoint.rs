//! Program entrypoint

use {
    crate::{error::FeatureGateError, processor},
    solana_account_info::AccountInfo,
    solana_address::Address,
    solana_msg::msg,
    solana_program_error::ProgramResult,
};

solana_program_entrypoint::entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Address,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = processor::process(program_id, accounts, instruction_data) {
        msg!(error.to_str::<FeatureGateError>());
        return Err(error);
    }
    Ok(())
}
