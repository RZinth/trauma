//! Hash verification functionality for downloads.
//!
//! This module provides hash type detection and verification capabilities
//! for downloaded files, supporting MD5 and CRC32 hash algorithms. It automatically
//! detects hash types based on format and provides verification against local files.
//!
//! # Supported Hash Types
//!
//! - **MD5**: 32-character hexadecimal strings (e.g., "d41d8cd98f00b204e9800998ecf8427e")
//! - **CRC32**: Numeric strings that can be parsed as u32 (e.g., "1127497")
//!
//! # Examples
//!
//! ## Hash Type Detection
//!
//! ```rust
//! use trauma::download::hash::{detect_hash_type, HashType};
//!
//! // MD5 hash detection
//! let md5_hash = "d41d8cd98f00b204e9800998ecf8427e";
//! assert_eq!(detect_hash_type(md5_hash), Some(HashType::Md5));
//!
//! // CRC32 hash detection
//! let crc32_hash = "1127497";
//! assert_eq!(detect_hash_type(crc32_hash), Some(HashType::Crc32));
//!
//! // Invalid hash
//! assert_eq!(detect_hash_type("invalid"), None);
//! ```
//!
//! ## File Verification
//!
//! ```rust,no_run
//! use trauma::download::hash::verify_hash;
//! use std::path::PathBuf;
//!
//! let file_path = PathBuf::from("downloaded_file.zip");
//! let expected_hash = Some("d41d8cd98f00b204e9800998ecf8427e".to_string());
//!
//! match verify_hash(&file_path, expected_hash.as_ref()) {
//!     Ok(true) => println!("Hash verification passed!"),
//!     Ok(false) => println!("Hash verification failed!"),
//!     Err(e) => println!("Error during verification: {}", e),
//! }
//! ```

use bacy::hash::{calculate_crc32, calculate_md5};
use std::error::Error;
use std::path::Path;

/// Supported hash types for file verification.
#[derive(Debug, Clone, PartialEq)]
pub enum HashType {
    /// MD5 hash algorithm
    Md5,
    /// CRC32 hash algorithm
    Crc32,
}

/// Detect hash type based on the hash string format.
///
/// MD5 hashes are 32 hex characters, CRC32 can be detected by trying to parse as number.
///
/// # Arguments
///
/// * `hash` - The hash string to analyze
///
/// # Returns
///
/// * `Some(HashType)` if the hash format is recognized
/// * `None` if the hash format is not recognized
///
/// # Examples
///
/// ```
/// use trauma::download::hash::{detect_hash_type, HashType};
///
/// // MD5 hash
/// assert_eq!(detect_hash_type("400a0698b5b8a84fc57ad96e0c3b57c3"), Some(HashType::Md5));
///
/// // CRC32 hash
/// assert_eq!(detect_hash_type("1127497"), Some(HashType::Crc32));
///
/// // Invalid hash
/// assert_eq!(detect_hash_type("invalid_hash"), None);
/// ```
pub fn detect_hash_type(hash: &str) -> Option<HashType> {
    if hash.len() == 32 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(HashType::Md5)
    } else if hash.parse::<u32>().is_ok() {
        Some(HashType::Crc32)
    } else {
        None
    }
}

/// Verify hash of a local file against an expected hash.
///
/// Returns true if hashes match or if no hash is provided.
///
/// # Arguments
///
/// * `file_path` - Path to the file to verify
/// * `expected_hash` - Optional expected hash string
///
/// # Returns
///
/// * `Ok(true)` if hashes match or no hash provided
/// * `Ok(false)` if file doesn't exist or hashes don't match
/// * `Err` if there's an error calculating the hash
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use trauma::download::hash::verify_hash;
///
/// let file_path = PathBuf::from("test_file.txt");
/// let expected_hash = Some("400a0698b5b8a84fc57ad96e0c3b57c3".to_string());
///
/// match verify_hash(&file_path, expected_hash.as_ref()) {
///     Ok(true) => println!("Hash verification passed"),
///     Ok(false) => println!("Hash verification failed"),
///     Err(e) => println!("Error verifying hash: {}", e),
/// }
/// ```
pub fn verify_hash(
    file_path: &Path,
    expected_hash: Option<&String>,
) -> Result<bool, Box<dyn Error>> {
    let Some(expected_hash) = expected_hash else {
        return Ok(true);
    };

    if !file_path.exists() {
        return Ok(false);
    }

    let hash_type = detect_hash_type(expected_hash);

    match hash_type {
        Some(HashType::Md5) => {
            let calculated_hash = calculate_md5(Path::new(file_path))?;
            Ok(calculated_hash.to_lowercase() == expected_hash.to_lowercase())
        }
        Some(HashType::Crc32) => {
            let calculated_hash = calculate_crc32(Path::new(file_path))?;
            let expected_crc32: u32 = expected_hash.parse().map_err(|_| "Invalid CRC32 format")?;
            Ok(calculated_hash == expected_crc32)
        }
        None => Ok(false),
    }
}
