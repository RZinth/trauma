//! Builder pattern implementation for creating Downloader instances.
//!
//! This module provides the [`DownloaderBuilder`] struct that implements the builder
//! pattern for configuring and creating [`Downloader`] instances. It allows for
//! flexible configuration of download behavior, progress display, HTTP settings,
//! and callback functions.
//!
//! # Examples
//!
//! ## Basic Builder Usage
//!
//! ```rust
//! use trauma::downloader::DownloaderBuilder;
//! use std::path::PathBuf;
//!
//! let downloader = DownloaderBuilder::new()
//!     .directory(PathBuf::from("./downloads"))
//!     .concurrent_downloads(5)
//!     .retries(3)
//!     .build();
//! ```
//!
//! ## Advanced Configuration with Callbacks
//!
//! ```rust
//! use trauma::downloader::DownloaderBuilder;
//! use trauma::download::Status;
//! use reqwest::header::{HeaderMap, USER_AGENT};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut headers = HeaderMap::new();
//! headers.insert(USER_AGENT, "MyApp/1.0".parse()?);
//!
//! let downloader = DownloaderBuilder::new()
//!     .headers(headers)
//!     .on_complete(|summary| {
//!         match summary.status() {
//!             Status::Success => println!("Successfully downloaded: {}", summary.download().filename),
//!             Status::Fail(msg) => println!("Failed to download {}: {}", summary.download().filename, msg),
//!             _ => {}
//!         }
//!     })
//!     .build();
//! # Ok(())
//! # }
//! ```
//!
//! ## Hidden Progress Bars
//!
//! ```rust
//! use trauma::downloader::DownloaderBuilder;
//!
//! // Create a downloader with no visible progress bars
//! let downloader = DownloaderBuilder::hidden().build();
//! ```

use super::{config::DownloaderConfig, downloader::Downloader};
use crate::download::Summary;
use crate::{ProgressBarOpts, StyleOptions};

use reqwest::header::{HeaderMap, HeaderValue, IntoHeaderName};
use std::{path::PathBuf, sync::Arc};

/// A builder used to create a [`Downloader`].
///
/// ```rust
/// # fn main()  {
/// use trauma::downloader::DownloaderBuilder;
///
/// let d = DownloaderBuilder::new().retries(5).directory("downloads".into()).build();
/// # }
/// ```
#[derive(Default)]
pub struct DownloaderBuilder {
    config: DownloaderConfig,
}

impl DownloaderBuilder {
    /// Creates a builder with the default options.
    pub fn new() -> Self {
        DownloaderBuilder::default()
    }

    /// Convenience function to hide the progress bars.
    pub fn hidden() -> Self {
        let mut builder = DownloaderBuilder::default();
        builder.config.style_options =
            StyleOptions::new(ProgressBarOpts::hidden(), ProgressBarOpts::hidden());
        builder
    }

    /// Sets the directory where to store the downloads.
    pub fn directory(mut self, directory: PathBuf) -> Self {
        self.config.directory = directory;
        self
    }

    /// Set the number of retries per download.
    pub fn retries(mut self, retries: u32) -> Self {
        self.config.retries = retries;
        self
    }

    /// Set the number of concurrent downloads.
    pub fn concurrent_downloads(mut self, concurrent_downloads: usize) -> Self {
        self.config.concurrent_downloads = concurrent_downloads;
        self
    }

    /// Set the downloader style options.
    pub fn style_options(mut self, style_options: StyleOptions) -> Self {
        self.config.style_options = style_options;
        self
    }

    /// Use range requests to get content length instead of HEAD requests.
    ///
    /// This is useful when servers don't provide accurate Content-Length headers
    /// in HEAD requests but do support range requests with Content-Range responses.
    pub fn use_range_for_content_length(mut self, use_range: bool) -> Self {
        self.config.use_range_for_content_length = use_range;
        self
    }

    /// Hide the main progress bar when downloading a single file.
    ///
    /// When enabled, only the individual file progress bar will be shown for single file downloads.
    /// The main progress bar will still be shown when downloading multiple files.
    pub fn single_file_progress(mut self, single_file: bool) -> Self {
        self.config.single_file_progress = single_file;
        self
    }

    /// Set callback for when each download completes.
    ///
    /// The callback will be called immediately when each download finishes,
    /// regardless of whether other downloads are still in progress.
    ///
    /// # Example
    ///
    /// ```rust
    /// use trauma::downloader::DownloaderBuilder;
    /// use trauma::download::Status;
    ///
    /// let downloader = DownloaderBuilder::new()
    ///     .on_complete(|summary| {
    ///         match summary.status() {
    ///             Status::Success => {
    ///                 println!("[Success] {} Downloaded", summary.download().filename);
    ///             }
    ///             Status::Fail(error) => {
    ///                 println!("[Failed] {} - Error: {}", summary.download().filename, error);
    ///             }
    ///             Status::Skipped(reason) => {
    ///                 println!("[Skipped] {} - {}", summary.download().filename, reason);
    ///             }
    ///             _ => {}
    ///         }
    ///     })
    ///     .build();
    /// ```
    pub fn on_complete<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Summary) + Send + Sync + 'static,
    {
        self.config.on_complete = Some(Arc::new(Box::new(callback)));
        self
    }

    /// Set whether to overwrite existing files.
    pub fn overwrite(mut self, overwrite: bool) -> Self {
        self.config.overwrite = overwrite;
        self
    }

    /// Helper method to get or create a new HeaderMap.
    fn new_header(&self) -> HeaderMap {
        match self.config.headers {
            Some(ref h) => h.to_owned(),
            _ => HeaderMap::new(),
        }
    }

    /// Add the http headers.
    ///
    /// You need to pass in a `HeaderMap`, not a `HeaderName`.
    /// `HeaderMap` is a set of http headers.
    ///
    /// You can call `.headers()` multiple times and all `HeaderMap` will be merged into a single one.
    ///
    /// # Example
    ///
    /// ```
    /// use reqwest::header::{self, HeaderValue, HeaderMap};
    /// use trauma::downloader::DownloaderBuilder;
    ///
    /// let ua = HeaderValue::from_str("curl/7.87").expect("Invalid UA");
    ///
    /// let builder = DownloaderBuilder::new()
    ///     .headers(HeaderMap::from_iter([(header::USER_AGENT, ua)]))
    ///     .build();
    /// ```
    ///
    /// See also [`header()`].
    ///
    /// [`header()`]: DownloaderBuilder::header
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        let mut new = self.new_header();
        new.extend(headers);

        self.config.headers = Some(new);
        self
    }

    /// Add the http header
    ///
    /// # Example
    ///
    /// You can use the `.header()` chain to add multiple headers
    ///
    /// ```
    /// use reqwest::header::{self, HeaderValue};
    /// use trauma::downloader::DownloaderBuilder;
    ///
    /// const FIREFOX_UA: &str =
    /// "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/109.0";
    ///
    /// let ua = HeaderValue::from_str(FIREFOX_UA).expect("Invalid UA");
    /// let auth = HeaderValue::from_str("Basic aGk6MTIzNDU2Cg==").expect("Invalid auth");
    ///
    /// let builder = DownloaderBuilder::new()
    ///     .header(header::USER_AGENT, ua)
    ///     .header(header::AUTHORIZATION, auth)
    ///     .build();
    /// ```
    ///
    /// If you need to pass in a `HeaderMap`, instead of calling `.header()` multiple times.
    /// See also [`headers()`].
    ///
    /// [`headers()`]: DownloaderBuilder::headers
    pub fn header<K: IntoHeaderName>(mut self, name: K, value: HeaderValue) -> Self {
        let mut new = self.new_header();

        new.insert(name, value);

        self.config.headers = Some(new);
        self
    }

    /// Create the [`Downloader`] with the specified options.
    pub fn build(self) -> Downloader {
        Downloader::new(self.config)
    }
}
