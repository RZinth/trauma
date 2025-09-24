//! Core download functionality.
//!
//! This module contains the [`Download`] struct and related functionality
//! for handling file downloads. It provides URL parsing, filename extraction,
//! hash verification, and download capability checking.
//!
//! # Examples
//!
//! ## Creating Downloads
//!
//! ```rust
//! use trauma::download::Download;
//! use std::convert::TryFrom;
//!
//! // Create from URL string (filename extracted automatically)
//! let download = Download::try_from("https://example.com/file.zip")?;
//! assert_eq!(download.filename, "file.zip");
//!
//! // Create with custom filename
//! let url = reqwest::Url::parse("https://example.com/download")?;
//! let download = Download::new(url, "custom-name.zip");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::Error;

use reqwest::{
    header::{ACCEPT_RANGES, CONTENT_LENGTH},
    Url,
};
use reqwest_middleware::ClientWithMiddleware;
use std::convert::TryFrom;
use std::error;
use std::path::Path;

/// Represents a file to be downloaded.
#[derive(Debug, Clone)]
pub struct Download {
    /// URL of the file to download.
    pub url: Url,
    /// File name used to save the file on disk.
    pub filename: String,
    /// Hash of the file (MD5 or CRC32).
    pub hash: Option<String>,
    /// Target file to extract from archives
    pub target_file: Option<String>,
}

impl Download {
    /// Creates a new [`Download`].
    ///
    /// When using the [`Download::try_from`] method, the file name is
    /// automatically extracted from the URL.
    ///
    /// ## Example
    ///
    /// The following calls are equivalent, minus some extra URL validations
    /// performed by `try_from`:
    ///
    /// ```no_run
    /// # use color_eyre::{eyre::Report, Result};
    /// use trauma::download::Download;
    /// use reqwest::Url;
    ///
    /// # fn main() -> Result<(), Report> {
    /// Download::try_from("https://example.com/file-0.1.2.zip")?;
    /// Download::new(Url::parse("https://example.com/file-0.1.2.zip")?, "file-0.1.2.zip");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(url: Url, filename: &str) -> Self {
        Self {
            url,
            filename: String::from(filename),
            hash: None,
            target_file: None,
        }
    }

    /// Calculate hash of local file and compare with expected hash.
    /// Returns true if hashes match or if no hash is provided.
    pub fn verify_hash(&self, file_path: &Path) -> Result<bool, Box<dyn error::Error>> {
        super::hash::verify_hash(file_path, self.hash.as_ref())
    }

    /// Check whether the download is resumable.
    pub async fn is_resumable(
        &self,
        client: &ClientWithMiddleware,
    ) -> Result<bool, reqwest_middleware::Error> {
        let res = client.head(self.url.clone()).send().await?;
        let headers = res.headers();
        match headers.get(ACCEPT_RANGES) {
            None => Ok(false),
            Some(x) if x == "none" => Ok(false),
            Some(_) => Ok(true),
        }
    }

    /// Retrieve the content_length of the download.
    ///
    /// Returns None if the "content-length" header is missing or if its value
    /// is not an u64.
    pub async fn content_length(
        &self,
        client: &ClientWithMiddleware,
    ) -> Result<Option<u64>, reqwest_middleware::Error> {
        let res = client.head(self.url.clone()).send().await?;
        let headers = res.headers();
        match headers.get(CONTENT_LENGTH) {
            None => Ok(None),
            Some(header_value) => match header_value.to_str() {
                Ok(v) => match v.to_string().parse::<u64>() {
                    Ok(v) => Ok(Some(v)),
                    Err(_) => Ok(None),
                },
                Err(_) => Ok(None),
            },
        }
    }

    /// Check if this download should use archive extraction.
    ///
    /// Returns true if a target file is specified for extraction.
    pub fn is_extraction(&self) -> bool {
        self.target_file.is_some()
    }

    /// Get the target file to extract from archive (if any).
    pub fn target_file(&self) -> Option<&str> {
        self.target_file.as_deref()
    }
}

impl TryFrom<&Url> for Download {
    type Error = crate::error::Error;

    fn try_from(value: &Url) -> Result<Self, Self::Error> {
        value
            .path_segments()
            .ok_or_else(|| {
                Error::InvalidUrl {
                    url: value.as_str().into(),
                    cause: "URL does not contain a valid path".into(),
                }
            })?
            .next_back()
            .map(String::from)
            .map(|filename| Download {
                url: value.clone(),
                filename: form_urlencoded::parse(filename.as_bytes())
                    .map(|(key, val)| [key, val].concat())
                    .collect(),
                hash: None,
                target_file: None,
            })
            .ok_or_else(|| {
                Error::InvalidUrl {
                    url: value.as_str().into(),
                    cause: "URL does not contain a filename".into(),
                }
            })
    }
}

impl TryFrom<&str> for Download {
    type Error = crate::error::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let url = Url::parse(value)
            .map_err(|e| {
                Error::InvalidUrl {
                    url: value.into(),
                    cause: Box::new(e),
                }
            })?;
        
        let filename = url
            .path_segments()
            .ok_or_else(|| {
                Error::InvalidUrl {
                    url: value.into(),
                    cause: "URL does not contain a valid path".into(),
                }
            })?
            .next_back()
            .ok_or_else(|| {
                Error::InvalidUrl {
                    url: value.into(),
                    cause: "URL does not contain a filename".into(),
                }
            })?;
        
        let decoded_filename = form_urlencoded::parse(filename.as_bytes())
            .map(|(key, val)| [key, val].concat())
            .collect();
        
        Ok(Download {
            url,
            filename: decoded_filename,
            hash: None,
            target_file: None,
        })
    }
}
