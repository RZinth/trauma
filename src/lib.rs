//! Trauma is crate aiming at providing a simple way to download files
//! asynchronously via HTTP(S).
//!
//! # Quick Start
//!
//! ```rust
//! use std::path::PathBuf;
//! use trauma::{download::Download, downloader::DownloaderBuilder, Error};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Error> {
//! let reqwest_rs = "https://github.com/seanmonstar/reqwest/archive/refs/tags/v0.11.9.zip";
//! let downloads = vec![Download::try_from(reqwest_rs)?];
//! let downloader = DownloaderBuilder::new()
//!     .directory(PathBuf::from("output"))
//!     .build();
//! downloader.download(&downloads).await;
//! # Ok(())
//! # }
//! ```
//!
//! # Module Organization
//!
//! The trauma crate is organized into several modules:
//!
//! - [`download`] - Core download functionality including the `Download` struct and hash verification
//! - [`downloader`] - The main `Downloader` and `DownloaderBuilder` for orchestrating downloads
//! - [`error`] - Centralized error handling with the `Error` enum
//! - [`http`] - HTTP client functionality and utilities
//! - [`progress`] - Progress bar styling and display management
//! - [`utils`] - Shared utility functions

pub mod archive;
pub mod download;
pub mod downloader;
pub mod error;
pub mod http;
pub mod progress;
pub mod utils;

pub use download::hash::{detect_hash_type, verify_hash, HashType};
pub use download::{Download, Status, Summary};
pub use downloader::{Downloader, DownloaderBuilder};
pub use error::{Error, Result};
pub use http::{create_http_client, HttpClientConfig};
pub use progress::{ProgressBarOpts, StyleOptions};
pub use utils::content_length::{
    extract_content_length, get_content_length, parse_content_range_total,
};
pub use archive::ZipFileInfo;
