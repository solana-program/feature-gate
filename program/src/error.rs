//! Program error types

use spl_program_error::*;

/// Program specific errors
#[spl_program_error]
pub enum FeatureGateError {
    /// Feature already activated
    #[error("Feature already activated")]
    FeatureAlreadyActivated,
    /// Incorrect staged features address
    #[error("Incorrect staged features address")]
    IncorrectStagedFeaturesAddress,
    /// Incorrect validator support signal address
    #[error("Incorrect validator support signal address")]
    IncorrectValidatorSupportSignalAddress,
    /// Feature already staged for activation
    #[error("Feature already staged for activation")]
    FeatureAlreadyStaged,
    /// Feature stage is full
    #[error("Feature stage is full")]
    FeatureStageFull,
}
