//! HTTP client setup and middleware configuration.
//!
//! This module provides HTTP client creation with comprehensive middleware
//! configuration including retry logic, tracing, proxy support, and custom headers.
//! It creates production-ready HTTP clients optimized for file downloads.
//!
//! # Features
//!
//! - **Retry Logic**: Exponential backoff retry policy for transient failures
//! - **Tracing**: Request/response logging and tracing integration
//! - **Proxy Support**: Optional HTTP/HTTPS proxy configuration
//! - **Custom Headers**: Default headers applied to all requests
//!
//! # Examples
//!
//! ## Basic Client Creation
//!
//! ```rust
//! use trauma::http::{create_http_client, HttpClientConfig};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = HttpClientConfig::default();
//! let client = create_http_client(config)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Client with Custom Configuration
//!
//! ```rust
//! use trauma::http::{create_http_client, HttpClientConfig};
//! use reqwest::header::{HeaderMap, USER_AGENT, ACCEPT};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut headers = HeaderMap::new();
//! headers.insert(USER_AGENT, "MyDownloader/1.0".parse()?);
//! headers.insert(ACCEPT, "*/*".parse()?);
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
//! ## Client with Proxy
//!
//! ```rust,no_run
//! use trauma::http::{create_http_client, HttpClientConfig};
//! use reqwest::Proxy;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let proxy = Proxy::http("http://proxy.example.com:8080")?;
//! let config = HttpClientConfig {
//!     retries: 3,
//!     proxy: Some(proxy),
//!     headers: None,
//! };
//!
//! let client = create_http_client(config)?;
//! # Ok(())
//! # }
//! ```

use reqwest::{header::HeaderMap, Proxy};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;

/// Configuration for HTTP client setup.
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Number of retries for failed requests.
    pub retries: u32,
    /// Optional proxy configuration.
    pub proxy: Option<Proxy>,
    /// Default headers to include with all requests.
    pub headers: Option<HeaderMap>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            retries: 3,
            proxy: None,
            headers: None,
        }
    }
}

/// Creates an HTTP client with middleware configuration.
///
/// This function sets up a reqwest client with:
/// - Tracing middleware for request/response logging
/// - Retry middleware with exponential backoff
/// - Optional proxy support
/// - Optional default headers
///
/// # Arguments
///
/// * `config` - Configuration for the HTTP client
///
/// # Returns
///
/// A configured `ClientWithMiddleware` ready for use
///
/// # Example
///
/// ```rust
/// use trauma::http::client::{create_http_client, HttpClientConfig};
///
/// let config = HttpClientConfig::default();
/// let client = create_http_client(config).unwrap();
/// ```
pub fn create_http_client(
    config: HttpClientConfig,
) -> Result<ClientWithMiddleware, reqwest::Error> {
    // Set up retry policy with exponential backoff
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(config.retries);

    // Build the inner reqwest client
    let mut inner_client_builder = reqwest::Client::builder();

    // Configure proxy if provided
    if let Some(proxy) = config.proxy {
        inner_client_builder = inner_client_builder.proxy(proxy);
    }

    // Configure default headers if provided
    if let Some(headers) = config.headers {
        inner_client_builder = inner_client_builder.default_headers(headers);
    }

    // Build the inner client
    let inner_client = inner_client_builder.build()?;

    // Build the client with middleware
    let client = ClientBuilder::new(inner_client)
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        .with(TracingMiddleware::default())
        // Retry failed requests.
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

    #[test]
    fn test_default_config() {
        let config = HttpClientConfig::default();
        assert_eq!(config.retries, 3);
        assert!(config.proxy.is_none());
        assert!(config.headers.is_none());
    }

    #[test]
    fn test_create_http_client_default() {
        let config = HttpClientConfig::default();
        let client = create_http_client(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_create_http_client_with_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("test-agent"));

        let config = HttpClientConfig {
            retries: 5,
            proxy: None,
            headers: Some(headers),
        };

        let client = create_http_client(config);
        assert!(client.is_ok());
    }
}
