//! Feature Gate program

#![deny(missing_docs)]
#![cfg_attr(not(test), forbid(unsafe_code))]

#[cfg(target_os = "solana")]
mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;

solana_pubkey::declare_id!("Feature111111111111111111111111111111111111");
