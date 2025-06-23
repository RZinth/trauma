//! Example showing cargo-style progress bar

use color_eyre::Result;
use std::path::PathBuf;
use trauma::downloader::DownloaderBuilder;
use trauma::download::Download;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Create a list of files to download
    let mut downloads = Vec::new();

    // Add some example files - using httpbin.org's streaming endpoint for reliable testing
    for i in 1..=20 {
        // Create unique URLs that each generate a file of approximately 256KB
        let url = format!("https://httpbin.org/bytes/256000?seed={}", i);
        downloads.push(Download::try_from(url.as_str())?);
    }

    // Create a downloader with cargo-style progress
    let downloader = DownloaderBuilder::new()
        .directory(PathBuf::from("downloads"))
        .cargo_style()
        .concurrent_downloads(5)  // limit concurrent downloads to better see the progress
        .build();

    // Start the downloads
    let results = downloader.download(&downloads).await;

    // Print summary
    println!("\nDownload complete! Downloaded {} files.", results.len());

    Ok(())
}
