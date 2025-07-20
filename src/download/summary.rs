//! Download summary functionality.
//!
//! This module contains the [`Summary`] struct and [`Status`] enum for tracking
//! download results and status. It provides comprehensive information about
//! download operations including success/failure status, file size, and HTTP details.
//!
//! # Examples
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
//! let mut summary = Summary::new(download, StatusCode::OK, 1024, true);
//!
//! // Check initial status
//! match summary.status() {
//!     Status::NotStarted => println!("Download not yet started"),
//!     Status::Success => println!("Download completed successfully"),
//!     Status::Fail(msg) => println!("Download failed: {}", msg),
//!     Status::Skipped(reason) => println!("Download skipped: {}", reason),
//!     Status::HashMismatch(details) => println!("Hash mismatch: {}", details),
//! }
//!
//! // Mark as failed
//! let failed_summary = summary.fail("Network timeout");
//! # Ok(())
//! # }
//! ```
//!
//! ## Creating and Modifying Summaries
//!
//! ```rust
//! use trauma::download::{Download, Summary, Status};
//! use reqwest::StatusCode;
//! use std::convert::TryFrom;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let download = Download::try_from("https://example.com/file.zip")?;
//!
//! // Create summary with initial status
//! let summary = Summary::new(download, StatusCode::OK, 2048, false)
//!     .with_status(Status::Success);
//!
//! println!("Downloaded {} bytes", summary.size());
//! println!("HTTP status: {}", summary.statuscode());
//! println!("Resumable: {}", summary.resumable());
//! # Ok(())
//! # }
//! ```

use super::download::Download;
use reqwest::StatusCode;

/// Download status enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Download failed with error message
    Fail(String),
    /// Download not yet started
    NotStarted,
    /// Download was skipped with reason
    Skipped(String),
    /// Download completed successfully
    Success,
    /// Download completed but hash verification failed
    HashMismatch(String),
}

/// Represents a [`Download`] summary.
#[derive(Debug, Clone)]
pub struct Summary {
    /// Downloaded items.
    download: Download,
    /// HTTP status code.
    statuscode: StatusCode,
    /// Download size in bytes.
    size: u64,
    /// Status.
    status: Status,
    /// Resumable.
    resumable: bool,
}

impl Summary {
    /// Create a new [`Download`] [`Summary`].
    pub fn new(download: Download, statuscode: StatusCode, size: u64, resumable: bool) -> Self {
        Self {
            download,
            statuscode,
            size,
            status: Status::NotStarted,
            resumable,
        }
    }

    /// Attach a status to a [`Download`] [`Summary`].
    pub fn with_status(self, status: Status) -> Self {
        Self { status, ..self }
    }

    /// Get the summary's status.
    pub fn statuscode(&self) -> StatusCode {
        self.statuscode
    }

    /// Get the summary's size.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get a reference to the summary's download.
    pub fn download(&self) -> &Download {
        &self.download
    }

    /// Get a reference to the summary's status.
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Mark the summary as failed with a message.
    pub fn fail(self, msg: impl std::fmt::Display) -> Self {
        Self {
            status: Status::Fail(format!("{}", msg)),
            ..self
        }
    }

    /// Mark the summary as skipped with a message.
    pub fn skip(self, msg: impl std::fmt::Display) -> Self {
        Self {
            status: Status::Skipped(format!("{}", msg)),
            ..self
        }
    }

    /// Mark the summary as having a hash mismatch with a message.
    pub fn hash_mismatch(self, msg: impl std::fmt::Display) -> Self {
        Self {
            status: Status::HashMismatch(format!("{}", msg)),
            ..self
        }
    }

    /// Set the summary's resumable.
    pub fn set_resumable(&mut self, resumable: bool) {
        self.resumable = resumable;
    }

    /// Get the summary's resumable.
    #[must_use]
    pub fn resumable(&self) -> bool {
        self.resumable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Url;

    fn create_test_download() -> Download {
        let url = Url::parse("http://example.com/test.zip").unwrap();
        Download::new(&url, "test.zip")
    }

    #[test]
    fn test_status_equality() {
        assert_eq!(Status::Success, Status::Success);
        assert_eq!(Status::NotStarted, Status::NotStarted);
        assert_eq!(
            Status::Fail("error".to_string()),
            Status::Fail("error".to_string())
        );
        assert_eq!(
            Status::Skipped("reason".to_string()),
            Status::Skipped("reason".to_string())
        );
        assert_eq!(
            Status::HashMismatch("hash".to_string()),
            Status::HashMismatch("hash".to_string())
        );

        assert_ne!(Status::Success, Status::NotStarted);
        assert_ne!(
            Status::Fail("error1".to_string()),
            Status::Fail("error2".to_string())
        );
    }

    #[test]
    fn test_summary_creation() {
        let download = create_test_download();
        let summary = Summary::new(download.clone(), StatusCode::OK, 1024, true);

        assert_eq!(summary.statuscode(), StatusCode::OK);
        assert_eq!(summary.size(), 1024);
        assert_eq!(summary.download().filename, "test.zip");
        assert_eq!(summary.status(), &Status::NotStarted);
        assert!(summary.resumable());
    }

    #[test]
    fn test_summary_with_status() {
        let download = create_test_download();
        let summary =
            Summary::new(download, StatusCode::OK, 1024, false).with_status(Status::Success);

        assert_eq!(summary.status(), &Status::Success);
        assert!(!summary.resumable());
    }

    #[test]
    fn test_summary_fail() {
        let download = create_test_download();
        let summary = Summary::new(download, StatusCode::INTERNAL_SERVER_ERROR, 0, false)
            .fail("Network error");

        match summary.status() {
            Status::Fail(msg) => assert_eq!(msg, "Network error"),
            _ => panic!("Expected Fail status"),
        }
    }

    #[test]
    fn test_summary_skip() {
        let download = create_test_download();
        let summary =
            Summary::new(download, StatusCode::OK, 1024, true).skip("File already exists");

        match summary.status() {
            Status::Skipped(msg) => assert_eq!(msg, "File already exists"),
            _ => panic!("Expected Skipped status"),
        }
    }

    #[test]
    fn test_summary_hash_mismatch() {
        let download = create_test_download();
        let summary = Summary::new(download, StatusCode::OK, 1024, false)
            .hash_mismatch("Expected hash abc123, got def456");

        match summary.status() {
            Status::HashMismatch(msg) => assert_eq!(msg, "Expected hash abc123, got def456"),
            _ => panic!("Expected HashMismatch status"),
        }
    }

    #[test]
    fn test_summary_set_resumable() {
        let download = create_test_download();
        let mut summary = Summary::new(download, StatusCode::OK, 1024, false);

        assert!(!summary.resumable());
        summary.set_resumable(true);
        assert!(summary.resumable());
    }

    #[test]
    fn test_status_debug_format() {
        let status = Status::Fail("test error".to_string());
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Fail"));
        assert!(debug_str.contains("test error"));
    }

    #[test]
    fn test_summary_debug_format() {
        let download = create_test_download();
        let summary = Summary::new(download, StatusCode::OK, 1024, true);
        let debug_str = format!("{:?}", summary);
        assert!(debug_str.contains("Summary"));
        assert!(debug_str.contains("test.zip"));
    }
}
