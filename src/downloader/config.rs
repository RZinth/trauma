//! Configuration structures and defaults for the downloader.
//!
//! This module provides configuration structures used by the [`Downloader`] and
//! [`DownloaderBuilder`]. It defines callback types, HTTP client configuration,
//! and the main downloader configuration structure with sensible defaults.
//!
//! # Examples
//!
//! ## Using Callbacks
//!
//! ```rust
//! use trauma::downloader::DownloadCallback;
//! use trauma::download::{Summary, Status};
//! use std::sync::Arc;
//!
//! // Create a callback function
//! let callback: DownloadCallback = Box::new(|summary: &Summary| {
//!     match summary.status() {
//!         Status::Success => println!("✓ Downloaded: {}", summary.download().filename),
//!         Status::Fail(msg) => println!("✗ Failed: {} - {}", summary.download().filename, msg),
//!         Status::HashMismatch(details) => println!("⚠ Hash mismatch: {} - {}", summary.download().filename, details),
//!         _ => println!("? Unknown status for: {}", summary.download().filename),
//!     }
//! });
//! ```
//!
//! ## HTTP Client Configuration
//!
//! ```rust
//! use trauma::http::HttpClientConfig;
//! use reqwest::header::{HeaderMap, USER_AGENT};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut headers = HeaderMap::new();
//! headers.insert(USER_AGENT, "MyDownloader/1.0".parse()?);
//!
//! let http_config = HttpClientConfig {
//!     retries: 5,
//!     proxy: None,
//!     headers: Some(headers),
//! };
//! # Ok(())
//! # }
//! ```

use crate::download::Summary;
use crate::StyleOptions;

use reqwest::header::HeaderMap;
use std::env::current_dir;
use std::sync::Arc;

/// Callback type for download completion events
pub type DownloadCallback = Box<dyn Fn(&Summary) + Send + Sync>;

/// Configuration for HTTP client setup
#[derive(Clone, Debug)]
pub struct HttpClientConfig {
    /// Number of retries per downloaded file.
    pub retries: u32,
    /// Optional proxy configuration.
    pub proxy: Option<reqwest::Proxy>,
    /// Custom HTTP headers.
    pub headers: Option<HeaderMap>,
}

/// Configuration structure for the downloader
#[derive(Clone)]
pub struct DownloaderConfig {
    /// Directory where to store the downloaded files.
    pub directory: std::path::PathBuf,
    /// Number of retries per downloaded file.
    pub retries: u32,
    /// Number of maximum concurrent downloads.
    pub concurrent_downloads: usize,
    /// Downloader style options.
    pub style_options: StyleOptions,
    /// Resume the download if necessary and possible.
    pub resumable: bool,
    /// Custom HTTP headers.
    pub headers: Option<HeaderMap>,
    /// Use range requests to get content length instead of HEAD requests.
    pub use_range_for_content_length: bool,
    /// Hide main progress bar for single file downloads.
    pub single_file_progress: bool,
    /// Callback for when each download completes.
    pub on_complete: Option<Arc<DownloadCallback>>,
    /// Force download and overwrite existing files.
    pub overwrite: bool,
}

impl std::fmt::Debug for DownloaderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloaderConfig")
            .field("directory", &self.directory)
            .field("retries", &self.retries)
            .field("concurrent_downloads", &self.concurrent_downloads)
            .field("style_options", &self.style_options)
            .field("resumable", &self.resumable)
            .field("headers", &self.headers)
            .field(
                "use_range_for_content_length",
                &self.use_range_for_content_length,
            )
            .field("single_file_progress", &self.single_file_progress)
            .field("on_complete", &self.on_complete.is_some())
            .field("overwrite", &self.overwrite)
            .finish()
    }
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        Self {
            directory: current_dir().unwrap_or_default(),
            retries: 3,
            concurrent_downloads: 32,
            style_options: StyleOptions::default(),
            resumable: true,
            headers: None,
            use_range_for_content_length: false,
            single_file_progress: false,
            on_complete: None,
            overwrite: false,
        }
    }
}
