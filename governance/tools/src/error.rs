//! Error types

use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the GovernanceTools
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum GovernanceToolsError {
    /// Account already initialized
    #[error("Account already initialized")]
    AccountAlreadyInitialized = 1100,

    /// Account doesn't exist
    #[error("Account doesn't exist")]
    AccountDoesNotExist, // 1101

    /// Invalid account owner
    #[error("Invalid account owner")]
    InvalidAccountOwner, // 1102

    /// Invalid Account type
    #[error("Invalid Account type")]
    InvalidAccountType, // 1103

    /// Creating account with pre-defined size but it's less than minimum necessary
    #[error(
        "Creating an account with prefetch space size too low to fit account data on creation"
    )]
    CreateAccountPrefetchSpaceExceeded, // 1104
}

impl PrintProgramError for GovernanceToolsError {
    fn print<E>(&self) {
        msg!("GOVERNANCE-TOOLS-ERROR: {}", &self.to_string());
    }
}

impl From<GovernanceToolsError> for ProgramError {
    fn from(e: GovernanceToolsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for GovernanceToolsError {
    fn type_of() -> &'static str {
        "Governance Tools Error"
    }
}
