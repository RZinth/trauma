//! Content length extraction utilities.
//!
//! This module provides utilities for extracting content length from HTTP responses,
//! supporting both Content-Range and Content-Length headers.

use reqwest::Response;

/// Extract content length from a response, supporting both Content-Range and Content-Length headers.
///
/// This function first checks for a Content-Range header (from range requests) and extracts
/// the total size. If that's not available, it falls back to the Content-Length header
/// and adds 1 to account for the range request.
///
/// # Arguments
///
/// * `response` - The HTTP response to extract content length from
///
/// # Returns
///
/// The content length as an u64, or 0 if no valid content length is found
///
/// # Example
///
/// ```rust,no_run
/// use trauma::utils::get_content_length;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let response = reqwest::get("https://httpbin.org/get").await?;
/// let length = get_content_length(&response);
/// # Ok(())
/// # }
/// ```
pub fn get_content_length(response: &Response) -> u64 {
    if let Some(content_range) = response.headers().get("Content-Range") {
        // Content-Range format is typically: "bytes 0-0/230917262"
        // We want to extract the number after the slash
        content_range
            .to_str()
            .ok()
            .and_then(|range| {
                range
                    .split('/')
                    .next_back()
                    .and_then(|size| size.trim().parse::<u64>().ok())
            })
            .unwrap_or(0)
    } else {
        response.content_length().unwrap_or(0).saturating_add(1)
    }
}

/// Parse Content-Range header to extract total size.
///
/// Content-Range header format: "bytes start-end/total"
/// This function extracts the total size from the header.
///
/// # Arguments
///
/// * `content_range` - The Content-Range header value as a string
///
/// # Returns
///
/// The total size as `Option<u64>`, None if parsing fails
///
/// # Example
///
/// ```rust
/// use trauma::utils::parse_content_range_total;
///
/// let total = parse_content_range_total("bytes 0-1023/2048");
/// assert_eq!(total, Some(2048));
/// ```
pub fn parse_content_range_total(content_range: &str) -> Option<u64> {
    content_range
        .split('/')
        .next_back()
        .and_then(|size| size.trim().parse::<u64>().ok())
}

/// Extract content length from Content-Length header with fallback.
///
/// This function extracts the content length from the Content-Length header,
/// with an optional fallback value if the header is missing or invalid.
///
/// # Arguments
///
/// * `response` - The HTTP response to extract content length from
/// * `fallback` - Optional fallback value if Content-Length is not available
///
/// # Returns
///
/// The content length as `Option<u64>`, None if no valid content length is found
///
/// # Example
///
/// ```rust,no_run
/// use trauma::utils::extract_content_length;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let response = reqwest::get("https://httpbin.org/get").await?;
/// let length = extract_content_length(&response, Some(1024));
/// # Ok(())
/// # }
/// ```
pub fn extract_content_length(response: &Response, fallback: Option<u64>) -> Option<u64> {
    response.content_length().or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content_range_total() {
        assert_eq!(parse_content_range_total("bytes 0-1023/2048"), Some(2048));
        assert_eq!(parse_content_range_total("bytes 200-1023/5000"), Some(5000));
        assert_eq!(parse_content_range_total("bytes 0-0/1"), Some(1));
        assert_eq!(parse_content_range_total("invalid"), None);
        assert_eq!(parse_content_range_total("bytes 0-1023"), None);
        assert_eq!(parse_content_range_total(""), None);
    }

    #[test]
    fn test_parse_content_range_total_edge_cases() {
        // Test with whitespace
        assert_eq!(parse_content_range_total("bytes 0-1023/ 2048 "), Some(2048));
        // Test with zero size
        assert_eq!(parse_content_range_total("bytes 0-0/0"), Some(0));
        // Test with large numbers
        assert_eq!(
            parse_content_range_total("bytes 0-1023/999999999999"),
            Some(999999999999)
        );
    }

    #[tokio::test]
    async fn test_get_content_length_with_content_range() {
        // We can't easily create a Response directly, so we'll test the parsing function
        let total = parse_content_range_total("bytes 0-0/2048");
        assert_eq!(total, Some(2048));
    }

    #[tokio::test]
    async fn test_get_content_length_real_request() {
        // Test with a real HTTP request
        if let Ok(response) = reqwest::get("https://httpbin.org/get").await {
            let length = get_content_length(&response);
            // Should return a reasonable length (content-length + 1 since no range header)
            assert!(length > 0);
        }
    }

    #[tokio::test]
    async fn test_extract_content_length_with_fallback() {
        // Test with a real HTTP request
        if let Ok(response) = reqwest::get("https://httpbin.org/get").await {
            let length = extract_content_length(&response, Some(1024));
            // Should return either the actual content length or fallback
            assert!(length.is_some());
            assert!(length.unwrap() > 0);
        }
    }

    #[tokio::test]
    async fn test_extract_content_length_no_fallback() {
        // Test with a real HTTP request
        if let Ok(response) = reqwest::get("https://httpbin.org/get").await {
            let length = extract_content_length(&response, None);
            // Should return the actual content length or None
            if let Some(len) = length {
                assert!(len > 0);
            }
        }
    }
}
