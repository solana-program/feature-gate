#[allow(deprecated)] // needed until Codama stops implementing deprecated traits
mod generated;

pub use generated::{programs::SOLANA_FEATURE_GATE_ID as ID, *};
