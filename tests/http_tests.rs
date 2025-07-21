//! Tests for HTTP module functionality.

use reqwest::header::{HeaderValue, USER_AGENT};
use trauma::http::client::{create_http_client, HttpClientConfig};

mod common;
use common::helpers::*;

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
    let config = create_test_http_config_with_retries(5);
    let client = create_http_client(config);
    assert!(client.is_ok());
}

#[test]
fn test_create_http_client_with_test_config() {
    let config = create_test_http_config();
    let client = create_http_client(config);
    assert!(client.is_ok());
}

#[test]
fn test_http_config_with_custom_headers() {
    let headers = create_test_headers_with_agent("custom-test-agent");
    let config = HttpClientConfig {
        retries: 2,
        proxy: None,
        headers: Some(headers.clone()),
    };

    assert_eq!(config.retries, 2);
    assert!(config.headers.is_some());
    assert_eq!(
        config.headers.unwrap().get(USER_AGENT),
        Some(&HeaderValue::from_static("custom-test-agent"))
    );
}