//! Core downloader implementation with fetch logic.
//!
//! This module contains the main [`Downloader`] struct that orchestrates
//! file downloads with support for concurrent downloads, progress reporting,
//! retry logic, resume capability, and hash verification.
//!
//! # Examples
//!
//! ## Basic Download
//!
//! ```rust,no_run
//! use trauma::downloader::DownloaderBuilder;
//! use trauma::download::Download;
//! use std::convert::TryFrom;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let downloader = DownloaderBuilder::new().build();
//! let downloads = vec![
//!     Download::try_from("https://example.com/file1.zip")?,
//!     Download::try_from("https://example.com/file2.pdf")?,
//! ];
//!
//! let summaries = downloader.download(&downloads).await;
//! for summary in summaries {
//!     println!("Downloaded: {} - Status: {:?}",
//!              summary.download().filename, summary.status());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Concurrent Downloads with Custom Directory
//!
//! ```rust,no_run
//! use trauma::downloader::DownloaderBuilder;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let downloader = DownloaderBuilder::new()
//!     .directory(PathBuf::from("./downloads"))
//!     .concurrent_downloads(10)
//!     .retries(5)
//!     .build();
//! # Ok(())
//! # }
//! ```

use super::config::DownloaderConfig;
use crate::download::{Download, Status, Summary};
use crate::http::{create_http_client, HttpClientConfig};
use crate::progress::display::ProgressDisplay;
use crate::utils::content_length::get_content_length;

use futures::stream::{self, StreamExt};
use reqwest::{
    header::{HeaderMap, RANGE},
    StatusCode,
};
use reqwest_middleware::ClientWithMiddleware;
use std::fmt;
use std::fmt::Debug;
use std::path::PathBuf;
use tokio::{fs, fs::OpenOptions, io::AsyncWriteExt};
use tracing::debug;

/// Represents the download controller.
///
/// A downloader can be created via its builder:
///
/// ```rust
/// # fn main()  {
/// use trauma::downloader::DownloaderBuilder;
///
/// let d = DownloaderBuilder::new().build();
/// # }
/// ```
#[derive(Clone)]
pub struct Downloader {
    config: DownloaderConfig,
}

impl Debug for Downloader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Downloader")
            .field("config", &self.config)
            .finish()
    }
}

impl Downloader {
    /// Creates a new Downloader with the given configuration.
    pub(crate) fn new(config: DownloaderConfig) -> Self {
        Self { config }
    }

    /// Gets the directory where files will be downloaded.
    pub fn directory(&self) -> &PathBuf {
        &self.config.directory
    }

    /// Gets the number of retries per download.
    pub fn retries(&self) -> u32 {
        self.config.retries
    }

    /// Gets the number of concurrent downloads.
    pub fn concurrent_downloads(&self) -> usize {
        self.config.concurrent_downloads
    }

    /// Gets whether downloads are resumable.
    pub fn resumable(&self) -> bool {
        self.config.resumable
    }

    /// Gets the custom headers.
    pub fn headers(&self) -> Option<&HeaderMap> {
        self.config.headers.as_ref()
    }

    /// Gets whether to use range requests for content length.
    pub fn use_range_for_content_length(&self) -> bool {
        self.config.use_range_for_content_length
    }

    /// Gets whether to show single file progress.
    pub fn single_file_progress(&self) -> bool {
        self.config.single_file_progress
    }

    /// Gets whether to overwrite existing files.
    pub fn overwrite(&self) -> bool {
        self.config.overwrite
    }

    /// Starts the downloads.
    pub async fn download(&self, downloads: &[Download]) -> Vec<Summary> {
        self.download_inner(downloads, None).await
    }

    /// Starts the downloads with proxy.
    pub async fn download_with_proxy(
        &self,
        downloads: &[Download],
        proxy: reqwest::Proxy,
    ) -> Vec<Summary> {
        self.download_inner(downloads, Some(proxy)).await
    }

    /// Starts the downloads.
    pub async fn download_inner(
        &self,
        downloads: &[Download],
        proxy: Option<reqwest::Proxy>,
    ) -> Vec<Summary> {
        // Prepare the HTTP client using the new HTTP module.
        let config = HttpClientConfig {
            retries: self.config.retries,
            proxy,
            headers: self.config.headers.clone(),
        };

        let client = create_http_client(config).unwrap();

        // Prepare the progress display.
        let progress_display = ProgressDisplay::new(
            self.config.style_options.clone(),
            downloads.len(),
            self.config.single_file_progress,
        );

        // Download the files asynchronously.
        let summaries = stream::iter(downloads)
            .map(|d| self.fetch(&client, d, &progress_display))
            .buffer_unordered(self.config.concurrent_downloads)
            .collect::<Vec<_>>()
            .await;

        // Finish the progress display.
        progress_display.finish();

        // Return the download summaries.
        summaries
    }

    /// Get content length using either HEAD request or Range request based on configuration.
    async fn get_content_length(
        &self,
        client: &ClientWithMiddleware,
        download: &Download,
    ) -> Result<Option<u64>, reqwest_middleware::Error> {
        if self.config.use_range_for_content_length {
            // Use range request to get content length
            let response = client
                .get(download.url.clone())
                .header("Range", "bytes=0-0")
                .send()
                .await?;

            Ok(Some(get_content_length(&response)))
        } else {
            // Use the original HEAD request method
            download.content_length(client).await
        }
    }

    /// Fetches the files and write them to disk.
    async fn fetch(
        &self,
        client: &ClientWithMiddleware,
        download: &Download,
        progress_display: &ProgressDisplay,
    ) -> Summary {
        let file_path = self.config.directory.join(&download.filename);

        // Check if file exists and hash matches
        if !self.config.overwrite && file_path.exists() {
            match download.verify_hash(&file_path) {
                Ok(true) => {
                    let file_size = fs::metadata(&file_path).await.map(|m| m.len()).unwrap_or(0);

                    return Summary::new(download.clone(), StatusCode::OK, file_size, false)
                        .skip("File exists with matching hash");
                }
                Ok(false) => {
                    // Hash verification failed - delete the file and trigger callback
                    let file_size = fs::metadata(&file_path).await.map(|m| m.len()).unwrap_or(0);

                    let hash_mismatch_summary =
                        Summary::new(download.clone(), StatusCode::OK, file_size, false)
                            .hash_mismatch("Hash mismatch, redownloading file");

                    // Call the callback for hash mismatch
                    if let Some(ref callback) = self.config.on_complete {
                        callback(&hash_mismatch_summary);
                    }

                    if let Err(e) = fs::remove_file(&file_path).await {
                        return Summary::new(
                            download.clone(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                            0,
                            false,
                        )
                        .fail(format!("Failed to remove file with wrong hash: {}", e));
                    }
                }
                Err(_) => {
                    // Error calculating hash, continue to download
                }
            }
        }

        // Create a download summary.
        let mut size_on_disk: u64 = 0;
        let mut can_resume = false;
        let output = self.config.directory.join(&download.filename);
        let mut summary = Summary::new(
            download.clone(),
            StatusCode::BAD_REQUEST,
            size_on_disk,
            can_resume,
        );
        let mut content_length: Option<u64> = None;

        // If resumable is turned on...
        if self.config.resumable {
            can_resume = match download.is_resumable(client).await {
                Ok(r) => r,
                Err(e) => {
                    let summary = summary.fail(e);
                    // Call the callback for failed downloads
                    if let Some(ref callback) = self.config.on_complete {
                        callback(&summary);
                    }
                    return summary;
                }
            };

            // Check if there is a file on disk already.
            if can_resume && output.exists() {
                debug!("A file with the same name already exists at the destination.");
                // If so, check file length to know where to restart the download from.
                size_on_disk = match output.metadata() {
                    Ok(m) => m.len(),
                    Err(e) => {
                        let summary = summary.fail(e);
                        // Call the callback for failed downloads
                        if let Some(ref callback) = self.config.on_complete {
                            callback(&summary);
                        }
                        return summary;
                    }
                };
            }

            // Update the summary accordingly.
            summary.set_resumable(can_resume);
        }

        // Always try to get content length regardless of resume status
        if content_length.is_none() {
            content_length = match self.get_content_length(client, download).await {
                Ok(l) => l,
                Err(e) => {
                    let summary = summary.fail(e);
                    // Call the callback for failed downloads
                    if let Some(ref callback) = self.config.on_complete {
                        callback(&summary);
                    }
                    return summary;
                }
            };
        }

        // Request the file.
        debug!("Fetching {}", &download.url);
        let mut req = client.get(download.url.clone());
        if self.config.resumable && can_resume {
            req = req.header(RANGE, format!("bytes={}-", size_on_disk));
        }

        // Add extra headers if needed.
        if let Some(ref h) = self.config.headers {
            req = req.headers(h.to_owned());
        }

        // Ensure there was no error while sending the request.
        let res = match req.send().await {
            Ok(res) => res,
            Err(e) => {
                let summary = summary.fail(e);
                // Call the callback for failed downloads
                if let Some(ref callback) = self.config.on_complete {
                    callback(&summary);
                }
                return summary;
            }
        };

        // Check wether or not we need to download the file.
        if let Some(content_length) = content_length {
            if content_length == size_on_disk {
                let summary = summary.with_status(Status::Skipped(
                    "the file was already fully downloaded".into(),
                ));
                // Call the callback for skipped downloads
                if let Some(ref callback) = self.config.on_complete {
                    callback(&summary);
                }
                return summary;
            }
        }

        // Check the status for errors.
        match res.error_for_status_ref() {
            Ok(_res) => (),
            Err(e) => {
                let summary = summary.fail(e);
                // Call the callback for failed downloads
                if let Some(ref callback) = self.config.on_complete {
                    callback(&summary);
                }
                return summary;
            }
        };

        // Update the summary with the collected details.
        let size = content_length.unwrap_or_else(|| {
            // If we still don't have content length, try to get it from the response
            get_content_length(&res)
        });
        let status = res.status();
        summary = Summary::new(download.clone(), status, size, can_resume);

        // If there is nothing else to download for this file, we can return.
        if size_on_disk > 0 && size == size_on_disk {
            let summary = summary.with_status(Status::Skipped(
                "the file was already fully downloaded".into(),
            ));
            // Call the callback for skipped downloads
            if let Some(ref callback) = self.config.on_complete {
                callback(&summary);
            }
            return summary;
        }

        // Create the progress bar.
        // If the download is being resumed, the progress bar position is
        // updated to start where the download stopped before.
        let pb = progress_display.create_child_progress(size, size_on_disk);

        // Prepare the destination directory/file.
        let output_dir = output.parent().unwrap_or(&output);
        debug!("Creating destination directory {:?}", output_dir);
        match fs::create_dir_all(output_dir).await {
            Ok(_res) => (),
            Err(e) => {
                let summary = summary.fail(e);
                // Call the callback for failed downloads
                if let Some(ref callback) = self.config.on_complete {
                    callback(&summary);
                }
                return summary;
            }
        };

        debug!("Creating destination file {:?}", &output);
        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .append(can_resume)
            .open(output)
            .await
        {
            Ok(file) => file,
            Err(e) => {
                let summary = summary.fail(e);
                // Call the callback for failed downloads
                if let Some(ref callback) = self.config.on_complete {
                    callback(&summary);
                }
                return summary;
            }
        };

        let mut final_size = size_on_disk;

        // Download the file chunk by chunk.
        debug!("Retrieving chunks...");
        let mut stream = res.bytes_stream();
        while let Some(item) = stream.next().await {
            // Retrieve chunk.
            let mut chunk = match item {
                Ok(chunk) => chunk,
                Err(e) => {
                    let summary = summary.fail(e);
                    // Call the callback for failed downloads
                    if let Some(ref callback) = self.config.on_complete {
                        callback(&summary);
                    }
                    return summary;
                }
            };
            let chunk_size = chunk.len() as u64;
            final_size += chunk_size;
            pb.inc(chunk_size);

            // Write the chunk to disk.
            match file.write_all_buf(&mut chunk).await {
                Ok(_res) => (),
                Err(e) => {
                    let summary = summary.fail(e);
                    // Call the callback for failed downloads
                    if let Some(ref callback) = self.config.on_complete {
                        callback(&summary);
                    }
                    return summary;
                }
            };
        }

        // Finish the progress bar once complete, and optionally remove it.
        progress_display.finish_child(pb);

        // Advance the main progress bar.
        progress_display.increment_main();

        // Create a new summary with the real download size and success status
        let summary = Summary::new(download.clone(), status, final_size, can_resume)
            .with_status(Status::Success);

        // Call the callback for successful downloads
        if let Some(ref callback) = self.config.on_complete {
            callback(&summary);
        }

        // Return the download summary.
        summary
    }
}
