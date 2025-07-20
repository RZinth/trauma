//! HTTP module containing HTTP client functionality.
//!
//! This module provides HTTP client setup, configuration, middleware,
//! and utility functions for handling HTTP operations. It handles client
//! creation with retry logic, tracing, proxy support, and content length extraction.
//!
//! # Overview
//!
//! The HTTP module is organized into two main components:
//!
//! - [`client`] - HTTP client creation and middleware configuration
//! - [`utils`] - HTTP utility functions for content length and header parsing
//!
//! # Examples
//!
//! ## Creating an HTTP Client
//!
//! ```rust
//! use trauma::http::{create_http_client, HttpClientConfig};
//! use reqwest::header::{HeaderMap, USER_AGENT};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create client with custom configuration
//! let mut headers = HeaderMap::new();
//! headers.insert(USER_AGENT, "MyApp/1.0".parse()?);
//!
//! let config = HttpClientConfig {
//!     retries: 5,
//!     proxy: None,
//!     headers: Some(headers),
//! };
//!
//! let client = create_http_client(config)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Content Length Extraction
//!
//! ```rust
//! use trauma::utils::get_content_length;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let response = reqwest::get("https://httpbin.org/get").await?;
//! let content_length = get_content_length(&response);
//! println!("Content length: {} bytes", content_length);
//! # Ok(())
//! # }
//! ```
//!
//! ## Parsing Content-Range Headers
//!
//! ```rust
//! use trauma::utils::parse_content_range_total;
//!
//! // Parse total size from Content-Range header
//! let content_range = "bytes 200-1023/1024";
//! if let Some(total) = parse_content_range_total(content_range) {
//!     println!("Total file size: {} bytes", total);
//! }
//! ```

pub mod client;

pub use client::{create_http_client, HttpClientConfig};
