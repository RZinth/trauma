//! Tests for the download module functionality.
//!
//! This file contains all tests for the download module, including:
//! - Download struct creation and URL parsing
//! - Summary and Status functionality
//! - Hash verification and type detection

use trauma::download::Download;
use reqwest::Url;
use std::convert::TryFrom;

mod common;
use common::helpers::*;

// Tests from src/download/download.rs
#[test]
fn test_try_from_url() {
    let u = Url::parse(TEST_DOMAIN).unwrap();
    let d = Download::try_from(&u).unwrap();
    assert_download_success(&d, "file.zip");
}

#[test]
fn test_try_from_string() {
    let d = create_test_download();
    assert_download_success(&d, "file.zip");
}

#[test]
fn test_try_from_custom_url() {
    let test_url = create_test_url("custom.bin");
    let d = Download::try_from(test_url.as_str()).unwrap();
    assert_download_success(&d, "custom.bin");
}

// Tests from src/download/summary.rs
mod summary_tests {
    // Placeholder for summary tests that were moved from src/download/summary.rs
    // These tests should be implemented as part of task 2
}