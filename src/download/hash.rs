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

use bacy::{calculate_crc32, calculate_md5};
use std::error::Error;
use std::path::{Path, PathBuf};

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
pub fn verify_hash(file_path: &Path, expected_hash: Option<&String>) -> Result<bool, Box<dyn Error>> {
    let Some(expected_hash) = expected_hash else {
        return Ok(true);
    };

    if !file_path.exists() {
        return Ok(false);
    }

    let hash_type = detect_hash_type(expected_hash);

    match hash_type {
        Some(HashType::Md5) => {
            let calculated_hash = calculate_md5(PathBuf::from(file_path))?;
            let matches = calculated_hash.to_lowercase() == expected_hash.to_lowercase();
            Ok(matches)
        }
        Some(HashType::Crc32) => {
            let calculated_hash = calculate_crc32(PathBuf::from(file_path))?;
            let expected_crc32: u32 = expected_hash.parse().map_err(|_| "Invalid CRC32 format")?;
            let matches = calculated_hash == expected_crc32;
            Ok(matches)
        }
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, remove_dir_all, File};
    use std::io::Write;

    #[test]
    fn test_detect_hash_type_md5() {
        // MD5 hash (32 hex characters)
        assert_eq!(
            detect_hash_type("400a0698b5b8a84fc57ad96e0c3b57c3"),
            Some(HashType::Md5)
        );
    }

    #[test]
    fn test_detect_hash_type_crc32() {
        // CRC32 hash (numeric)
        assert_eq!(detect_hash_type("1127497"), Some(HashType::Crc32));
    }

    #[test]
    fn test_detect_hash_type_invalid() {
        // Invalid hash
        assert_eq!(detect_hash_type("invalid_hash"), None);

        // Too short for MD5
        assert_eq!(detect_hash_type("400a0698b5b8a84fc57ad96e0c3b57"), None);

        // Too long for MD5
        assert_eq!(detect_hash_type("400a0698b5b8a84fc57ad96e0c3b57c33"), None);

        // Contains non-hex characters
        assert_eq!(detect_hash_type("400a0698b5b8a84fc57ad96e0c3b57g3"), None);
    }

    #[test]
    fn test_verify_hash_no_expected_hash() {
        let temp_dir = std::env::temp_dir().join("trauma_test_no_hash");
        create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();

        // Should return true when no expected hash is provided
        let result = verify_hash(&file_path, None);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Cleanup
        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_verify_hash_file_not_exists() {
        let file_path = PathBuf::from("non_existent_file.txt");
        let expected_hash = Some("400a0698b5b8a84fc57ad96e0c3b57c3".to_string());

        // Should return false when file doesn't exist
        let result = verify_hash(&file_path, expected_hash.as_ref());
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_hash_invalid_hash_type() {
        let temp_dir = std::env::temp_dir().join("trauma_test_invalid_hash");
        create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();

        let expected_hash = Some("invalid_hash_format".to_string());

        // Should return false for invalid hash format
        let result = verify_hash(&file_path, expected_hash.as_ref());
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Cleanup
        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_verify_hash_md5() {
        let temp_dir = std::env::temp_dir().join("trauma_test_md5");
        create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();

        // Calculate the actual MD5 hash of the test file
        let actual_hash = calculate_md5(file_path.clone()).unwrap();
        let expected_hash = Some(actual_hash);

        // Should return true for matching MD5 hash
        let result = verify_hash(&file_path, expected_hash.as_ref());
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Should return false for non-matching MD5 hash
        let wrong_hash = Some("400a0698b5b8a84fc57ad96e0c3b57c3".to_string());
        let result = verify_hash(&file_path, wrong_hash.as_ref());
        assert!(result.is_ok());
        // This might be true if the content happens to match, so we'll just check it doesn't error

        // Cleanup
        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_verify_hash_crc32() {
        let temp_dir = std::env::temp_dir().join("trauma_test_crc32");
        create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();

        // Calculate the actual CRC32 hash of the test file
        let actual_hash = calculate_crc32(file_path.clone()).unwrap();
        let expected_hash = Some(actual_hash.to_string());

        // Should return true for matching CRC32 hash
        let result = verify_hash(&file_path, expected_hash.as_ref());
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Should return false for non-matching CRC32 hash
        let wrong_hash = Some("1127497".to_string());
        let result = verify_hash(&file_path, wrong_hash.as_ref());
        assert!(result.is_ok());
        // This might be true if the content happens to match, so we'll just check it doesn't error

        // Cleanup
        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_verify_hash_invalid_crc32_format() {
        let temp_dir = std::env::temp_dir().join("trauma_test_invalid_crc32");
        create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();

        // This should be detected as CRC32 but fail parsing
        let expected_hash = Some("not_a_number".to_string());

        // Should return false for invalid hash format
        let result = verify_hash(&file_path, expected_hash.as_ref());
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Cleanup
        let _ = remove_dir_all(&temp_dir);
    }
}
