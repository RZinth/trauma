//! Error handling for the Trauma library.
//!
//! This module provides centralized error handling with comprehensive error types
//! that can occur during download operations. All errors implement the standard
//! Error trait and provide detailed context about failures.

use std::io;
use thiserror::Error;

/// Errors that can happen when using Trauma.
///
/// This enum represents all possible errors that can occur during download operations,
/// providing detailed context and proper error chaining for debugging and error handling.
#[derive(Error, Debug)]
pub enum Error {
    /// Error from an underlying system.
    ///
    /// This variant captures internal errors that don't fit into other categories,
    /// typically representing unexpected system-level failures.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Error from the underlying URL parser or the expected URL format.
    ///
    /// This variant is returned when a provided URL cannot be parsed or doesn't
    /// conform to the expected format for HTTP/HTTPS downloads.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// I/O Error.
    ///
    /// This variant wraps standard I/O errors that can occur during file operations,
    /// such as creating, writing, or reading files during the download process.
    #[error("I/O error")]
    IOError {
        #[from]
        source: io::Error,
    },

    /// Error from the Reqwest library.
    ///
    /// This variant wraps HTTP client errors from the reqwest library, including
    /// network failures, HTTP status errors, and request/response processing errors.
    #[error("Reqwest Error")]
    Reqwest {
        #[from]
        source: reqwest::Error,
    },
}

/// Result type alias for operations that can fail with a Trauma error.
///
/// This type alias provides a convenient way to return results from Trauma operations
/// without having to specify the full `Result<T, Error>` type signature.
pub type Result<T> = std::result::Result<T, Error>;
