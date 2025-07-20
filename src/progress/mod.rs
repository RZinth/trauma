//! Progress module containing progress bar functionality.
//!
//! This module provides progress bar styling, display management,
//! and progress reporting functionality for download operations. It handles
//! both individual file progress and overall download progress coordination.
//!
//! # Overview
//!
//! The progress module is organized into two main components:
//!
//! - `style` - Progress bar styling options and templates
//! - `display` - Progress bar display management and coordination
//!
//! # Examples
//!
//! ## Custom Progress Bar Styling
//!
//! ```rust
//! use trauma::progress::{StyleOptions, ProgressBarOpts};
//!
//! // Create custom style options
//! let style_options = StyleOptions::new(
//!     ProgressBarOpts::new(
//!         Some("[{bar:40.cyan/blue}] {pos}/{len} {msg}".to_string()),
//!         Some("█▉▊▋▌▍▎▏  ".to_string()),
//!         true,
//!         false
//!     ),
//!     ProgressBarOpts::with_pip_style(),
//! );
//! ```
//!
//! ## Using with Downloader
//!
//! ```rust
//! use trauma::downloader::DownloaderBuilder;
//! use trauma::progress::StyleOptions;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let downloader = DownloaderBuilder::new()
//!     .style_options(StyleOptions::default())
//!     .build();
//! # Ok(())
//! # }
//! ```
//!
//! ## Hidden Progress Bars
//!
//! ```rust
//! use trauma::progress::{StyleOptions, ProgressBarOpts};
//!
//! // Create style options with hidden progress bars
//! let hidden_style = StyleOptions::new(
//!     ProgressBarOpts::hidden(),
//!     ProgressBarOpts::hidden(),
//! );
//! ```

pub(crate) mod display;
pub(crate) mod style;

pub use display::ProgressDisplay;
pub use style::{ProgressBarOpts, StyleOptions};