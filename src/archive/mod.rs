//! Archive extraction functionality.
//!
//! This module provides functionality to extract specific files from remote archives
//! without downloading the entire archive, significantly reducing bandwidth usage.

pub mod zip;

pub use zip::{ZipExtractor, ZipFileInfo};
