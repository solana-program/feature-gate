//! Program error types.

use {
    num_derive::FromPrimitive,
    solana_program::{
        decode_error::DecodeError,
        msg,
        program_error::{PrintProgramError, ProgramError},
    },
    thiserror::Error,
};

/// Program error types.
// Note: Shank does not export the type when we use `spl_program_error`.
#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum FeatureGateError {
    /// Feature already activated
    #[error("Feature already activated")]
    FeatureAlreadyActivated,
}

impl PrintProgramError for FeatureGateError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<FeatureGateError> for ProgramError {
    fn from(e: FeatureGateError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for FeatureGateError {
    fn type_of() -> &'static str {
        "FeatureGateError"
    }
}
