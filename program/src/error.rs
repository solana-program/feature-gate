//! Program error types.

use {
    num_derive::FromPrimitive,
    num_enum::TryFromPrimitive,
    solana_program_error::{ProgramError, ToStr},
    thiserror::Error,
};

/// Program error types.
// Note: Shank does not export the type when we use `spl_program_error`.
#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum FeatureGateError {
    /// Feature already activated
    #[error("Feature already activated")]
    FeatureAlreadyActivated,
}

impl ToStr for FeatureGateError {
    fn to_str(&self) -> &'static str {
        match self {
            FeatureGateError::FeatureAlreadyActivated => "Feature already activated",
        }
    }
}

impl From<FeatureGateError> for ProgramError {
    fn from(e: FeatureGateError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
