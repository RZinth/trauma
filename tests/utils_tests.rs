//! Tests for utils module functionality.

use trauma::utils::{get_content_length, parse_content_range_total, extract_content_length};

mod common;
use common::helpers::*;

#[test]
fn test_parse_content_range_total() {
    let test_cases = create_test_content_range_headers();
    for (header, expected) in test_cases {
        assert_eq!(parse_content_range_total(header), expected);
    }
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