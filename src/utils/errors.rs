//! Error Types for RPC Service
//!
//! This module defines error types used throughout the RPC service:
//! - Input validation errors (InvalidDecimals, InvalidSymbol, etc.)
//! - Network communication errors (InvalidUrl, InvalidGitUrl)
//! - Token generation errors (ProgramModified, ContractModified)
//! - File system operation errors (FileIoError, InvalidPath)
//! - Git operation errors (GitError, ClonedRepoNotFound)
//!
//! Each error type includes context about what went wrong and how to
//! potentially fix the issue.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// Enum representing various errors that can occur during token generation
#[derive(Error, Debug, Deserialize, Serialize)]
pub enum TokenGenErrors {
    // Error indicating no `.move` files were found in the given path
    #[error("Invalid path: No .move file found")]
    InvalidPathNoMoveFiles,

    // Error indicating that the given contract has been modified unexpectedly
    #[error("Given contract is modified")]
    ProgramModified,

    // Error indicating that the provided decimals value is invalid
    #[error("Invalid decimals provided")]
    InvalidDecimals,

    // Error indicating that the provided symbol for the token is invalid
    #[error("Invalid symbol provided")]
    InvalidSymbol,

    // Error indicating that the provided name for the token is invalid
    #[error("Invalid name provided")]
    InvalidName,

    // Error indicating that the provided description for the token is invalid
    #[error("Invalid description provided")]
    InvalidDescription,

    // Error indicating that the provided URL is not valid
    #[error("The provided URL is not a valid URL.")]
    InvalidUrl,

    // Error indicating that the provided URL is not a valid Git URL
    #[error("The provided URL is not a valid Git URL.")]
    InvalidGitUrl,

    // Error indicating that the repository name could not be extracted from the provided URL
    #[error("Failed to extract repository name.")]
    InvalidRepo,

    // Error indicating a content mismatch between expected and actual contract content
    #[error("Content mismatch detected")]
    ContractModified,

    // Error indicating that the cloned repository could not be found
    #[error("Cloned repo not found")]
    ClonedRepoNotFound,

    // Error indicating a failure during a Git operation, with the error message passed as an argument
    #[error("Git operation failed: {0}")]
    GitError(String),

    // Error indicating a file I/O operation failure, with the error message passed as an argument
    #[error("File I/O error: {0}")]
    FileIoError(String),

    // General error for verification failure, with the error message passed as an argument
    #[error("{0}")]
    VerifyResultError(String),

    // General error with a custom message passed as an argument
    #[error("An error occurred: {0}")]
    GeneralError(String),

    // Error indicating that the given path is invalid, with the invalid path passed as an argument
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}
