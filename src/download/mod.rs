//! Download module containing download-related functionality.
//!
//! This module provides structures and functions for handling downloads,
//! including the core Download struct, summary reporting, and hash verification.
//! It serves as the foundation for all download operations in the trauma crate.
//!
//! # Overview
//!
//! The download module is organized into three main components:
//!
//! - [`download`] - Core Download struct and URL handling
//! - [`summary`] - Download result tracking and status reporting  
//! - [`hash`] - File integrity verification through hash checking
//!
//! # Examples
//!
//! ## Creating a Download
//!
//! ```rust
//! use trauma::download::Download;
//! use std::convert::TryFrom;
//!
//! // Create a download from a URL string
//! let download = Download::try_from("https://example.com/file.zip")?;
//! println!("Downloading: {}", download.filename);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Working with Download Status
//!
//! ```rust
//! use trauma::download::{Status, Summary, Download};
//! use reqwest::StatusCode;
//! use std::convert::TryFrom;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let download = Download::try_from("https://example.com/file.zip")?;
//! let summary = Summary::new(download, StatusCode::OK, 1024, true);
//!
//! // Check download status
//! match summary.status() {
//!     Status::Success => println!("Download completed successfully"),
//!     Status::Fail(msg) => println!("Download failed: {}", msg),
//!     Status::HashMismatch(expected) => println!("Hash verification failed, expected: {}", expected),
//!     _ => println!("Download in progress or skipped"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Hash Verification
//!
//! ```rust
//! use trauma::download::{HashType, detect_hash_type, verify_hash};
//! use std::path::PathBuf;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Detect hash type and verify file
//! let hash = "d41d8cd98f00b204e9800998ecf8427e"; // MD5 hash
//! if let Some(_hash_type) = detect_hash_type(hash) {
//!     let file_path = PathBuf::from("downloaded_file.txt");
//!     let is_valid = verify_hash(&file_path, Some(&hash.to_string()))?;
//!     println!("Hash verification: {}", if is_valid { "passed" } else { "failed" });
//! }
//! # Ok(())
//! # }
//! ```

pub mod download;
pub mod hash;
pub mod summary;

pub use download::Download;
pub use hash::{detect_hash_type, verify_hash, HashType};
pub use summary::{Status, Summary};
