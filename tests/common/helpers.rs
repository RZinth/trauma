use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use trauma::{Download, DownloaderBuilder};
use trauma::progress::{ProgressBarOpts, StyleOptions};
use trauma::HttpClientConfig;

// Common test constants
pub const TEST_DOMAIN: &str = "http://domain.com/file.zip";
pub const TEST_HTTPBIN_URL: &str = "https://httpbin.org/bytes/1024";
pub const TEST_USER_AGENT: &str = "trauma-test-agent";

/// Creates a temporary directory for testing purposes
pub fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Creates a temporary file with the given content
pub fn create_temp_file(dir: &Path, filename: &str, content: &[u8]) -> PathBuf {
    let file_path = dir.join(filename);
    fs::write(&file_path, content).expect("Failed to write temporary file");
    file_path
}

/// Creates a test URL for download testing
pub fn create_test_url(filename: &str) -> String {
    format!("https://example.com/{}", filename)
}

/// Creates test file content of specified size
pub fn create_test_content(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

/// Asserts that a file exists at the given path
pub fn assert_file_exists(path: &Path) {
    assert!(path.exists(), "File should exist at path: {:?}", path);
}

/// Asserts that a file has the expected size
pub fn assert_file_size(path: &Path, expected_size: u64) {
    let metadata = fs::metadata(path).expect("Failed to get file metadata");
    assert_eq!(
        metadata.len(),
        expected_size,
        "File size mismatch at path: {:?}",
        path
    );
}

/// Creates a mock HTTP response for testing
pub fn create_mock_response_headers() -> Vec<(String, String)> {
    vec![
        ("content-length".to_string(), "1024".to_string()),
        ("content-type".to_string(), "application/octet-stream".to_string()),
    ]
}

/// Helper function to create test download configuration
pub fn create_test_config() -> trauma::DownloaderBuilder {
    trauma::DownloaderBuilder::new()
        .directory(create_temp_dir().path().to_path_buf())
        .concurrent_downloads(2)
}

// === Download Creation Helpers ===

/// Creates a test download from the common test domain
pub fn create_test_download() -> Download {
    Download::try_from(TEST_DOMAIN).expect("Failed to create test download")
}

/// Creates a test download with custom filename
pub fn create_test_download_with_filename(url: &str, filename: &str) -> Download {
    let mut download = Download::try_from(url).expect("Failed to create download");
    download.filename = filename.to_string();
    download
}

/// Creates multiple test downloads for batch testing
pub fn create_test_downloads(count: usize) -> Vec<Download> {
    (0..count)
        .map(|i| {
            let url = format!("https://httpbin.org/bytes/{}", 256 * (i + 1));
            Download::try_from(url.as_str()).expect("Failed to create download")
        })
        .collect()
}

// === HTTP Configuration Helpers ===

/// Creates test headers with common user agent
pub fn create_test_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(TEST_USER_AGENT));
    headers
}

/// Creates test headers with custom user agent
pub fn create_test_headers_with_agent(agent: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(agent).expect("Invalid header value"));
    headers
}

/// Creates a test HTTP client configuration
pub fn create_test_http_config() -> HttpClientConfig {
    HttpClientConfig {
        retries: 3,
        proxy: None,
        headers: Some(create_test_headers()),
    }
}

/// Creates a test HTTP client configuration with custom retries
pub fn create_test_http_config_with_retries(retries: u32) -> HttpClientConfig {
    HttpClientConfig {
        retries,
        proxy: None,
        headers: Some(create_test_headers()),
    }
}

// === Progress Bar Helpers ===

/// Creates default test progress bar options
pub fn create_test_progress_opts() -> ProgressBarOpts {
    ProgressBarOpts::new(None, None, true, false)
}

/// Creates hidden progress bar options for testing
pub fn create_hidden_progress_opts() -> ProgressBarOpts {
    ProgressBarOpts::hidden()
}

/// Creates pip-style progress bar options for testing
pub fn create_pip_style_progress_opts() -> ProgressBarOpts {
    ProgressBarOpts::with_pip_style()
}

/// Creates custom progress bar options with template and chars
pub fn create_custom_progress_opts(template: &str, chars: &str) -> ProgressBarOpts {
    ProgressBarOpts::new(
        Some(template.to_string()),
        Some(chars.to_string()),
        true,
        false
    )
}

/// Creates default test style options
pub fn create_test_style_options() -> StyleOptions {
    let main = create_test_progress_opts();
    let child = create_pip_style_progress_opts();
    StyleOptions::new(main, child)
}

/// Creates disabled style options for testing
pub fn create_disabled_style_options() -> StyleOptions {
    let main = create_hidden_progress_opts();
    let child = create_hidden_progress_opts();
    StyleOptions::new(main, child)
}

// === Downloader Builder Helpers ===

/// Creates a basic test downloader builder
pub fn create_test_downloader_builder() -> DownloaderBuilder {
    DownloaderBuilder::new()
}

/// Creates a test downloader builder with common test configuration
pub fn create_configured_test_downloader_builder(temp_dir: &Path) -> DownloaderBuilder {
    DownloaderBuilder::new()
        .directory(temp_dir.to_path_buf())
        .concurrent_downloads(2)
        .retries(1)
        .headers(create_test_headers())
}

/// Creates a test downloader builder with all options configured
pub fn create_full_test_downloader_builder(temp_dir: &Path) -> DownloaderBuilder {
    DownloaderBuilder::new()
        .directory(temp_dir.to_path_buf())
        .concurrent_downloads(4)
        .retries(2)
        .headers(create_test_headers())
        .use_range_for_content_length(true)
        .single_file_progress(true)
        .overwrite(true)
}

// === Assertion Helpers ===

/// Asserts that a download result is successful
pub fn assert_download_success(download: &Download, expected_filename: &str) {
    assert_eq!(download.filename, expected_filename);
}

/// Asserts that progress bar options are configured correctly
pub fn assert_progress_opts_enabled(opts: &ProgressBarOpts) {
    let pb = opts.clone().to_progress_bar(100);
    assert!(!pb.is_hidden(), "Progress bar should be enabled");
}

/// Asserts that progress bar options are disabled
pub fn assert_progress_opts_disabled(opts: &ProgressBarOpts) {
    let pb = opts.clone().to_progress_bar(100);
    assert!(pb.is_hidden(), "Progress bar should be disabled");
}

/// Asserts that style options are enabled
pub fn assert_style_options_enabled(style: &StyleOptions) {
    assert!(style.is_enabled(), "Style options should be enabled");
}

/// Asserts that style options are disabled
pub fn assert_style_options_disabled(style: &StyleOptions) {
    assert!(!style.is_enabled(), "Style options should be disabled");
}

// === Hash Testing Helpers ===

/// Common test hash values for different types
pub const TEST_MD5_HASH: &str = "d41d8cd98f00b204e9800998ecf8427e";
pub const TEST_SHA1_HASH: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
pub const TEST_SHA256_HASH: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// Creates test content with known hash values
pub fn create_empty_test_content() -> Vec<u8> {
    Vec::new() // Empty content has known hash values
}

/// Creates test content for hash verification
pub fn create_hello_world_content() -> Vec<u8> {
    b"Hello, World!".to_vec()
}

// === Content Range Testing Helpers ===

/// Creates test content range headers for testing
pub fn create_test_content_range_headers() -> Vec<(&'static str, Option<u64>)> {
    vec![
        ("bytes 0-1023/2048", Some(2048)),
        ("bytes 200-1023/5000", Some(5000)),
        ("bytes 0-0/1", Some(1)),
        ("invalid", None),
        ("bytes 0-1023", None),
        ("", None),
    ]
}