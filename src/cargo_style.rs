//! Provides a cargo-style progress bar implementation for trauma downloads

use crate::download::Download;
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;

/// Style for cargo-like progress display with a downloaded items list
pub struct CargoProgressStyle {
    /// Maximum number of files to display in the list
    max_displayed_files: usize,
    /// List of downloaded files (filename, is_completed)
    downloaded_files: Vec<(String, bool)>,
    /// Multi progress bar
    multi: Arc<MultiProgress>,
    /// Main progress bar for total progress
    main_bar: Arc<ProgressBar>,
    /// Active download progress bars (filename -> progress bar)
    active_bars: std::collections::HashMap<String, ProgressBar>,
}

impl CargoProgressStyle {
    /// Creates a new cargo-style progress display
    pub fn new(total_files: usize) -> Self {
        let multi = Arc::new(MultiProgress::new());

        // Create the main progress bar that shows at the bottom
        let main_bar = multi.add(ProgressBar::new(total_files as u64));
        main_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {pos}/{len} files downloaded")
                .unwrap()
                .progress_chars("━━━")  // Unicode box-drawing characters
        );

        Self {
            max_displayed_files: 15,  // Show at most 15 downloaded files
            downloaded_files: Vec::with_capacity(total_files),
            multi,
            main_bar: Arc::new(main_bar),
            active_bars: std::collections::HashMap::new(),
        }
    }

    /// Adds a new download to the list
    pub fn add_download(&mut self, download: &Download) -> ProgressBar {
        // Add to our tracking list
        self.downloaded_files.push((download.filename.clone(), false));

        // Create a progress bar for the current download
        let pb = self.multi.add(ProgressBar::new(0));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {bar:40.cyan/blue} {bytes}/{total_bytes} {bytes_per_sec} {msg:.cyan}")
                .unwrap()
                .progress_chars("━━━")  // Unicode box-drawing characters
        );
        pb.set_message(format!("{}...", download.filename));

            // Store this progress bar in our active bars
            self.active_bars.insert(download.filename.clone(), pb.clone());

        // Redraw the display to show the updated list
        self.redraw();

        pb
    }

    /// Mark a download as completed
    pub fn complete_download(&mut self, filename: &str) {
        // Mark the download as completed in our list
        if let Some(idx) = self.downloaded_files.iter().position(|(name, _)| name == filename) {
            self.downloaded_files[idx].1 = true;
        }

        // Advance the main progress bar
        self.main_bar.inc(1);

        // Finish the progress bar for this file
        if let Some(bar) = self.active_bars.remove(filename) {
            bar.finish_and_clear();
        }

        // Display the completed download
        eprintln!("  {}", style(format!("Downloaded {}", filename)).green());

        // Redraw the display for remaining active downloads
        self.redraw();
    }

    /// Redraw the entire display
    fn redraw(&self) {
        // We don't need to clear or redraw completed downloads - they stay in terminal history
        // and are only printed when explicitly completed

        // We only need to update the active progress bars
        // The MultiProgress handles positioning of the active bars

        // The main progress bar will show at the bottom after any active downloads
        self.main_bar.tick();

        // For active downloads, ensure they're properly updated
        for bar in self.active_bars.values() {
            bar.tick();
        }
    }

    /// Set total bytes for a specific download progress bar
    pub fn set_total_bytes(&self, download_name: &str, bytes: u64) {
        if let Some(bar) = self.active_bars.get(download_name) {
            bar.set_length(bytes);
        }
    }

    /// Finish the progress display
    pub fn finish(&mut self) {
        // Clear all active progress bars
        for bar in self.active_bars.values() {
            bar.finish_and_clear();
        }
        self.active_bars.clear();

        // Finish the main progress bar
        self.main_bar.finish_and_clear();
    }
}
