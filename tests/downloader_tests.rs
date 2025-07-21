//! Tests for the downloader module functionality.
//!
//! This file contains all tests for the downloader module, including tests for:
//! - Core Downloader functionality
//! - DownloaderBuilder pattern
//! - HttpClientConfig

use trauma::downloader::DownloaderBuilder;

use reqwest::header::{HeaderValue, USER_AGENT};
use std::sync::{atomic, Arc};

mod common;
use common::helpers::*;

// Tests moved from src/downloader/downloader.rs
#[test]
fn test_downloader_creation() {
    let downloader = create_test_downloader_builder().build();

    assert_eq!(downloader.retries(), 3);
    assert_eq!(downloader.concurrent_downloads(), 32);
    assert!(downloader.resumable());
    assert!(!downloader.use_range_for_content_length());
    assert!(!downloader.single_file_progress());
    assert!(!downloader.overwrite());
}

#[test]
fn test_downloader_getters() {
    let temp_dir = create_temp_dir();
    let downloader = DownloaderBuilder::new()
        .directory(temp_dir.path().to_path_buf())
        .retries(5)
        .concurrent_downloads(10)
        .use_range_for_content_length(true)
        .single_file_progress(true)
        .overwrite(true)
        .build();

    assert_eq!(downloader.directory(), temp_dir.path());
    assert_eq!(downloader.retries(), 5);
    assert_eq!(downloader.concurrent_downloads(), 10);
    assert!(downloader.use_range_for_content_length());
    assert!(downloader.single_file_progress());
    assert!(downloader.overwrite());
}

#[test]
fn test_downloader_debug() {
    let downloader = DownloaderBuilder::new().build();
    let debug_str = format!("{:?}", downloader);

    assert!(debug_str.contains("Downloader"));
    assert!(debug_str.contains("config"));
}

#[test]
fn test_downloader_clone() {
    let downloader = DownloaderBuilder::new().build();
    let cloned = downloader.clone();

    assert_eq!(downloader.retries(), cloned.retries());
    assert_eq!(
        downloader.concurrent_downloads(),
        cloned.concurrent_downloads()
    );
    assert_eq!(downloader.resumable(), cloned.resumable());
}

// Tests moved from src/downloader/builder.rs
#[test]
fn test_builder_defaults() {
    let downloader = DownloaderBuilder::new().build();

    assert_eq!(downloader.retries(), 3);
    assert_eq!(downloader.concurrent_downloads(), 32);
    assert!(downloader.resumable());
    assert!(!downloader.use_range_for_content_length());
    assert!(!downloader.single_file_progress());
    assert!(!downloader.overwrite());
    assert!(downloader.headers().is_none());
}

#[test]
fn test_builder_configuration() {
    let temp_dir = create_temp_dir();
    let downloader = DownloaderBuilder::new()
        .directory(temp_dir.path().to_path_buf())
        .retries(5)
        .concurrent_downloads(10)
        .use_range_for_content_length(true)
        .single_file_progress(true)
        .overwrite(true)
        .build();

    assert_eq!(downloader.directory(), temp_dir.path());
    assert_eq!(downloader.retries(), 5);
    assert_eq!(downloader.concurrent_downloads(), 10);
    assert!(downloader.use_range_for_content_length());
    assert!(downloader.single_file_progress());
    assert!(downloader.overwrite());
}

#[test]
fn test_builder_headers() {
    let headers = create_test_headers();
    let downloader = DownloaderBuilder::new().headers(headers.clone()).build();

    assert!(downloader.headers().is_some());
    assert_eq!(
        downloader.headers().unwrap().get(USER_AGENT),
        Some(&HeaderValue::from_static(TEST_USER_AGENT))
    );
}

#[test]
fn test_builder_single_header() {
    let downloader = DownloaderBuilder::new()
        .header(USER_AGENT, HeaderValue::from_static("single-test-agent"))
        .build();

    assert!(downloader.headers().is_some());
    assert_eq!(
        downloader.headers().unwrap().get(USER_AGENT),
        Some(&HeaderValue::from_static("single-test-agent"))
    );
}

#[test]
fn test_builder_hidden() {
    let downloader = DownloaderBuilder::hidden().build();
    
    assert_eq!(downloader.retries(), 3);
    assert_eq!(downloader.concurrent_downloads(), 32);
}

#[test]
fn test_builder_on_complete_callback() {
    let callback_called = Arc::new(atomic::AtomicBool::new(false));
    let callback_called_clone = callback_called.clone();

    let _downloader = DownloaderBuilder::new()
        .on_complete(move |_summary| {
            callback_called_clone.store(true, atomic::Ordering::SeqCst);
        })
        .build();
}

#[test]
fn test_builder_chaining() {
    let temp_dir = create_temp_dir();
    let headers = create_test_headers_with_agent("chained-agent");

    let downloader = DownloaderBuilder::new()
        .directory(temp_dir.path().to_path_buf())
        .retries(10)
        .concurrent_downloads(5)
        .headers(headers)
        .use_range_for_content_length(true)
        .single_file_progress(true)
        .overwrite(true)
        .build();

    assert_eq!(downloader.directory(), temp_dir.path());
    assert_eq!(downloader.retries(), 10);
    assert_eq!(downloader.concurrent_downloads(), 5);
    assert!(downloader.use_range_for_content_length());
    assert!(downloader.single_file_progress());
    assert!(downloader.overwrite());
    assert!(downloader.headers().is_some());
}

// Tests moved from src/downloader/config.rs
#[test]
fn test_http_client_config() {
    let config = create_test_http_config_with_retries(5);
    
    assert_eq!(config.retries, 5);
    assert!(config.proxy.is_none());
    assert!(config.headers.is_some());
}

#[test]
fn test_http_client_config_default() {
    let config = create_test_http_config();
    
    assert_eq!(config.retries, 3);
    assert!(config.proxy.is_none());
    assert!(config.headers.is_some());
}