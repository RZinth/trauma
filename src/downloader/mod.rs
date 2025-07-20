//! Downloader module containing core downloader functionality, builder pattern, and configuration.
//!
//! This module provides the main [`Downloader`] struct and its associated builder pattern
//! for configuring and executing file downloads. It handles concurrent downloads, progress
//! reporting, retry logic, and callback management.
//!
//! # Overview
//!
//! The downloader module is organized into three main components:
//!
//! - `downloader` - Core Downloader struct with download orchestration logic
//! - `builder` - DownloaderBuilder for flexible configuration using the builder pattern
//! - `config` - Configuration structures and callback types
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use trauma::downloader::DownloaderBuilder;
//! use trauma::download::Download;
//! use std::convert::TryFrom;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a downloader with default settings
//! let downloader = DownloaderBuilder::new().build();
//!
//! // Create downloads
//! let downloads = vec![
//!     Download::try_from("https://example.com/file1.zip")?,
//!     Download::try_from("https://example.com/file2.pdf")?,
//! ];
//!
//! // Execute downloads
//! let summaries = downloader.download(&downloads).await;
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Configuration
//!
//! ```rust
//! use trauma::downloader::DownloaderBuilder;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let downloader = DownloaderBuilder::new()
//!     .directory(PathBuf::from("./downloads"))
//!     .concurrent_downloads(5)
//!     .retries(3)
//!     .on_complete(|summary| {
//!         println!("Downloaded: {}", summary.download().filename);
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
//! // Create a downloader with hidden progress bars
//! let downloader = DownloaderBuilder::hidden().build();
//! ```

pub mod builder;
pub mod config;
pub mod downloader;

pub use builder::DownloaderBuilder;
pub use config::{DownloadCallback, HttpClientConfig};
pub use downloader::Downloader;