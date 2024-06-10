//! Program error types

use spl_program_error::*;

/// Program specific errors
#[spl_program_error]
pub enum FeatureGateError {
    /// Feature already activated
    #[error("Feature already activated")]
    FeatureAlreadyActivated,
}
