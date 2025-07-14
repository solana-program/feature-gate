//! Feature Gate program

#![deny(missing_docs)]
#![cfg_attr(not(test), forbid(unsafe_code))]

#[cfg(all(target_os = "solana", feature = "bpf-entrypoint"))]
mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;

solana_pubkey::declare_id!("Feature111111111111111111111111111111111111");
