//! Represents a file to be downloaded.

use crate::Error;
use reqwest::{
    header::{ACCEPT_RANGES, CONTENT_LENGTH},
    StatusCode, Url,
};
use reqwest_middleware::ClientWithMiddleware;
use std::convert::TryFrom;
use std::path::PathBuf;
use bacy::{calculate_md5, calculate_crc32};

/// Represents a file to be downloaded.
#[derive(Debug, Clone)]
pub struct Download {
    /// URL of the file to download.
    pub url: Url,
    /// File name used to save the file on disk.
    pub filename: String,
    /// Hash of the file (MD5 or CRC32).
    pub hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HashType {
    Md5,
    Crc32,
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
    /// Download::new(&Url::parse("https://example.com/file-0.1.2.zip")?, "file-0.1.2.zip");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(url: &Url, filename: &str) -> Self {
        Self {
            url: url.clone(),
            filename: String::from(filename),
            hash: None,
        }
    }

    /// Creates a new [`Download`] with hash.
    pub fn new_with_hash(url: &Url, filename: &str, hash: Option<String>) -> Self {
        Self {
            url: url.clone(),
            filename: String::from(filename),
            hash,
        }
    }

    /// Detect hash type based on the hash string format.
    /// MD5 hashes are 32 hex characters, CRC32 can be detected by trying to parse as number.
    pub fn detect_hash_type(hash: &str) -> Option<HashType> {
        if hash.len() == 32 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
            Some(HashType::Md5)
        } else if hash.parse::<u32>().is_ok() {
            Some(HashType::Crc32)
        } else {
            None
        }
    }

    /// Calculate hash of local file and compare with expected hash.
    /// Returns true if hashes match or if no hash is provided.
    pub fn verify_hash(&self, file_path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
        let Some(expected_hash) = &self.hash else {
            return Ok(true);
        };

        if !file_path.exists() {
            return Ok(false);
        }

        match Self::detect_hash_type(expected_hash) {
            Some(HashType::Md5) => {
                let calculated_hash = calculate_md5(file_path.clone())?;
                Ok(calculated_hash.to_lowercase() == expected_hash.to_lowercase())
            }
            Some(HashType::Crc32) => {
                let calculated_hash = calculate_crc32(file_path.clone())?;
                let expected_crc32: u32 = expected_hash.parse()
                    .map_err(|_| "Invalid CRC32 format")?;
                Ok(calculated_hash == expected_crc32)
            }
            None => {
                // Unknown hash format, consider it invalid
                Ok(false)
            }
        }
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
    /// is not a u64.
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
}

impl TryFrom<&Url> for Download {
    type Error = crate::Error;

    fn try_from(value: &Url) -> Result<Self, Self::Error> {
        value
            .path_segments()
            .ok_or_else(|| {
                Error::InvalidUrl(format!(
                    "the url \"{}\" does not contain a valid path",
                    value
                ))
            })?
            .next_back()
            .map(String::from)
            .map(|filename| Download {
                url: value.clone(),
                filename: form_urlencoded::parse(filename.as_bytes())
                    .map(|(key, val)| [key, val].concat())
                    .collect(),
                hash: None,
            })
            .ok_or_else(|| {
                Error::InvalidUrl(format!("the url \"{}\" does not contain a filename", value))
            })
    }
}

impl TryFrom<&str> for Download {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Url::parse(value)
            .map_err(|e| {
                Error::InvalidUrl(format!("the url \"{}\" cannot be parsed: {}", value, e))
            })
            .and_then(|u| Download::try_from(&u))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Fail(String),
    NotStarted,
    Skipped(String),
    Success,
}

/// Represents a [`Download`] summary.
#[derive(Debug, Clone)]
pub struct Summary {
    /// Downloaded items.
    download: Download,
    /// HTTP status code.
    statuscode: StatusCode,
    /// Download size in bytes.
    size: u64,
    /// Status.
    status: Status,
    /// Resumable.
    resumable: bool,
}

impl Summary {
    /// Create a new [`Download`] [`Summary`].
    pub fn new(download: Download, statuscode: StatusCode, size: u64, resumable: bool) -> Self {
        Self {
            download,
            statuscode,
            size,
            status: Status::NotStarted,
            resumable,
        }
    }

    /// Attach a status to a [`Download`] [`Summary`].
    pub fn with_status(self, status: Status) -> Self {
        Self { status, ..self }
    }

    /// Get the summary's status.
    pub fn statuscode(&self) -> StatusCode {
        self.statuscode
    }

    /// Get the summary's size.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get a reference to the summary's download.
    pub fn download(&self) -> &Download {
        &self.download
    }

    /// Get a reference to the summary's status.
    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn fail(self, msg: impl std::fmt::Display) -> Self {
        Self {
            status: Status::Fail(format!("{}", msg)),
            ..self
        }
    }

    pub fn skip(self, msg: impl std::fmt::Display) -> Self {
        Self {
            status: Status::Skipped(format!("{}", msg)),
            ..self
        }
    }

    /// Set the summary's resumable.
    pub fn set_resumable(&mut self, resumable: bool) {
        self.resumable = resumable;
    }

    /// Get the summary's resumable.
    #[must_use]
    pub fn resumable(&self) -> bool {
        self.resumable
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const DOMAIN: &str = "http://domain.com/file.zip";

    #[test]
    fn test_try_from_url() {
        let u = Url::parse(DOMAIN).unwrap();
        let d = Download::try_from(&u).unwrap();
        assert_eq!(d.filename, "file.zip")
    }

    #[test]
    fn test_try_from_string() {
        let d = Download::try_from(DOMAIN).unwrap();
        assert_eq!(d.filename, "file.zip")
    }

    #[test]
    fn test_detect_hash_type() {
        // MD5 hash (32 hex characters)
        assert_eq!(
            Download::detect_hash_type("400a0698b5b8a84fc57ad96e0c3b57c3"),
            Some(HashType::Md5)
        );
        
        // CRC32 hash (numeric)
        assert_eq!(
            Download::detect_hash_type("1127497"),
            Some(HashType::Crc32)
        );
        
        // Invalid hash
        assert_eq!(
            Download::detect_hash_type("invalid_hash"),
            None
        );
    }
}