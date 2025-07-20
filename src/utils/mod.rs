//! Shared utility functions.
//!
//! This module contains utility functions that are used across multiple
//! modules in the trauma crate. It provides common functionality for
//! content length extraction and HTTP response processing.
//!
//! # Overview
//!
//! The utils module currently contains:
//!
//! - [`content_length`] - Content length extraction from HTTP responses
//!
//! # Examples
//!
//! ## Extracting Content Length
//!
//! ```rust
//! use trauma::utils::get_content_length;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let response = reqwest::get("https://httpbin.org/get").await?;
//! let length = get_content_length(&response);
//! println!("Response content length: {} bytes", length);
//! # Ok(())
//! # }
//! ```
//!
//! ## Parsing Content-Range Headers
//!
//! ```rust
//! use trauma::utils::parse_content_range_total;
//!
//! // Extract total size from a Content-Range header
//! let header_value = "bytes 0-1023/2048";
//! if let Some(total_size) = parse_content_range_total(header_value) {
//!     println!("Total file size: {} bytes", total_size);
//! }
//! ```

pub mod content_length;

// Re-export commonly used utilities
pub use content_length::{extract_content_length, get_content_length, parse_content_range_total};