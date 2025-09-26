//! ZIP file extraction implementation.
//!
//! This module provides functionality to extract specific files from remote ZIP
//! archives using HTTP range requests, avoiding the need to download entire archives.

use crate::error::Error;
use reqwest_middleware::ClientWithMiddleware;
use reqwest::Url;

const EOCD_SIGNATURE: &[u8; 4] = b"\x50\x4b\x05\x06";
const CENTRAL_DIR_SIGNATURE: &[u8; 4] = b"\x50\x4b\x01\x02";

const COMPRESSION_STORED: u16 = 0;
const COMPRESSION_DEFLATE: u16 = 8;

const EOCD_MIN_SIZE: usize = 22;
const CENTRAL_DIR_ENTRY_MIN_SIZE: usize = 46;
const LOCAL_HEADER_MIN_SIZE: usize = 30;

const EOCD_SEARCH_SIZE: u64 = 65536;

/// Information about a file within a ZIP archive.
#[derive(Debug, Clone)]
pub struct ZipFileInfo {
    pub compression_method: u16,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub local_header_offset: u64,
}

/// ZIP file extractor that can extract specific files from remote ZIP archives.
pub struct ZipExtractor<'a> {
    client: &'a ClientWithMiddleware,
    url: &'a Url,
    zip_size: u64,
}

impl<'a> ZipExtractor<'a> {
    pub async fn new(client: &'a ClientWithMiddleware, url: &'a Url) -> Result<Self, Error> {
        let head_response = client.head(url.clone()).send().await
            .map_err(|e| Error::Archive {
                message: "Failed to get ZIP file info".into(),
                cause: Some(Box::new(e)),
            })?;
        
        let zip_size = head_response
            .headers()
            .get("content-length")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| Error::Archive {
                message: "Could not determine ZIP file size".into(),
                cause: None,
            })?;

        let accept_ranges = head_response
            .headers()
            .get("accept-ranges")
            .map(|h| h.to_str().unwrap_or(""))
            .unwrap_or("");

        if !accept_ranges.contains("bytes") {
            return Err(Error::Archive {
                message: "Server doesn't support range requests".into(),
                cause: None,
            });
        }

        Ok(Self {
            client,
            url,
            zip_size,
        })
    }

    /// Extract a specific file from the ZIP archive.
    pub async fn extract_file(&self, target_filename: &str) -> Result<Vec<u8>, Error> {
        let eocd_size = std::cmp::min(EOCD_SEARCH_SIZE, self.zip_size);
        let eocd_start = self.zip_size - eocd_size;
        
        let eocd_response = self.client
            .get(self.url.as_str())
            .header("Range", format!("bytes={}-{}", eocd_start, self.zip_size - 1))
            .send()
            .await
            .map_err(|e| Error::Archive {
                message: "Failed to download EOCD".into(),
                cause: Some(Box::new(e)),
            })?;
        
        let eocd_data = eocd_response.bytes().await
            .map_err(|e| Error::Archive {
                message: "Failed to read EOCD data".into(),
                cause: Some(Box::new(e)),
            })?;

        let eocd_offset = eocd_data.windows(4)
            .rposition(|window| window == EOCD_SIGNATURE)
            .ok_or_else(|| Error::Archive {
                message: "Could not find End of Central Directory Record".into(),
                cause: None,
            })?;

        let eocd = &eocd_data[eocd_offset..];
        if eocd.len() < EOCD_MIN_SIZE {
            return Err(Error::Archive {
                message: "Invalid EOCD record".into(),
                cause: None,
            });
        }

        let cd_size = u32::from_le_bytes([eocd[12], eocd[13], eocd[14], eocd[15]]) as u64;
        let cd_offset = u32::from_le_bytes([eocd[16], eocd[17], eocd[18], eocd[19]]) as u64;

        let cd_data = if eocd_start + eocd_offset as u64 >= cd_offset + cd_size {
            let cd_start_in_eocd = (eocd_offset as u64 + eocd_start) - cd_offset - cd_size;
            eocd_data[cd_start_in_eocd as usize..eocd_offset].to_vec()
        } else {
            let cd_response = self.client
                .get(self.url.as_str())
                .header("Range", format!("bytes={}-{}", cd_offset, cd_offset + cd_size - 1))
                .send()
                .await
                .map_err(|e| Error::Archive {
                    message: "Failed to download central directory".into(),
                    cause: Some(Box::new(e)),
                })?;
            
            cd_response.bytes().await
                .map_err(|e| Error::Archive {
                    message: "Failed to read central directory".into(),
                    cause: Some(Box::new(e)),
                })?
                .to_vec()
        };

        let file_info = self.find_file_in_central_directory(&cd_data, target_filename)?
            .ok_or_else(|| Error::Archive {
                message: format!("File '{}' not found in ZIP", target_filename).into(),
                cause: None,
            })?;

        let header_response = self.client
            .get(self.url.as_str())
            .header("Range", format!("bytes={}-{}", file_info.local_header_offset, file_info.local_header_offset + 29))
            .send()
            .await
            .map_err(|e| Error::Archive {
                message: "Failed to download local file header".into(),
                cause: Some(Box::new(e)),
            })?;

        let header_data = header_response.bytes().await
            .map_err(|e| Error::Archive {
                message: "Failed to read local file header".into(),
                cause: Some(Box::new(e)),
            })?;

        if header_data.len() < LOCAL_HEADER_MIN_SIZE {
            return Err(Error::Archive {
                message: "Invalid local file header".into(),
                cause: None,
            });
        }

        let filename_length = u16::from_le_bytes([header_data[26], header_data[27]]) as u64;
        let extra_field_length = u16::from_le_bytes([header_data[28], header_data[29]]) as u64;
        let data_start = file_info.local_header_offset + LOCAL_HEADER_MIN_SIZE as u64 + filename_length + extra_field_length;

        let data_end = data_start + file_info.compressed_size - 1;
        let file_response = self.client
            .get(self.url.as_str())
            .header("Range", format!("bytes={}-{}", data_start, data_end))
            .send()
            .await
            .map_err(|e| Error::Archive {
                message: "Failed to download file data".into(),
                cause: Some(Box::new(e)),
            })?;

        let compressed_data = file_response.bytes().await
            .map_err(|e| Error::Archive {
                message: "Failed to read file data".into(),
                cause: Some(Box::new(e)),
            })?;

        match file_info.compression_method {
            COMPRESSION_STORED => {
                Ok(compressed_data.to_vec())
            }
            COMPRESSION_DEFLATE => {
                use flate2::read::DeflateDecoder;
                use std::io::Read;

                let mut decoder = DeflateDecoder::new(&compressed_data[..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)
                    .map_err(|e| Error::Archive {
                        message: "Deflate decompression failed".into(),
                        cause: Some(Box::new(e)),
                    })?;
                Ok(decompressed)
            }
            method => Err(Error::UnsupportedCompression { 
                message: method,
                cause: None,
            }),
        }
    }

    /// Parse central directory to find specific file info.
    fn find_file_in_central_directory(&self, cd_data: &[u8], target_filename: &str) -> Result<Option<ZipFileInfo>, Error> {
        let mut offset = 0;

        while offset + CENTRAL_DIR_ENTRY_MIN_SIZE <= cd_data.len() {
            if &cd_data[offset..offset + 4] != CENTRAL_DIR_SIGNATURE {
                break;
            }

            let compression_method = u16::from_le_bytes([cd_data[offset + 10], cd_data[offset + 11]]);
            let compressed_size = u32::from_le_bytes([
                cd_data[offset + 20], cd_data[offset + 21], 
                cd_data[offset + 22], cd_data[offset + 23]
            ]) as u64;
            let uncompressed_size = u32::from_le_bytes([
                cd_data[offset + 24], cd_data[offset + 25], 
                cd_data[offset + 26], cd_data[offset + 27]
            ]) as u64;
            let filename_length = u16::from_le_bytes([cd_data[offset + 28], cd_data[offset + 29]]) as usize;
            let extra_field_length = u16::from_le_bytes([cd_data[offset + 30], cd_data[offset + 31]]) as usize;
            let comment_length = u16::from_le_bytes([cd_data[offset + 32], cd_data[offset + 33]]) as usize;
            let local_header_offset = u32::from_le_bytes([
                cd_data[offset + 42], cd_data[offset + 43], 
                cd_data[offset + 44], cd_data[offset + 45]
            ]) as u64;

            let filename_start = offset + CENTRAL_DIR_ENTRY_MIN_SIZE;
            if filename_start + filename_length > cd_data.len() {
                break;
            }

            let filename = String::from_utf8_lossy(&cd_data[filename_start..filename_start + filename_length]);

            if filename == target_filename {
                return Ok(Some(ZipFileInfo {
                    compression_method,
                    compressed_size,
                    uncompressed_size,
                    local_header_offset,
                }));
            }

            offset += CENTRAL_DIR_ENTRY_MIN_SIZE + filename_length + extra_field_length + comment_length;
        }

        Ok(None)
    }

    /// Check if a URL likely points to a ZIP file.
    pub fn is_likely_zip_url(url: &Url) -> bool {
        url.path().to_lowercase().ends_with(".zip")
    }
}
