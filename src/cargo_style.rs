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
    /// Current active download progress bar
    current_bar: Option<ProgressBar>,
}

impl CargoProgressStyle {
    /// Creates a new cargo-style progress display
    pub fn new(total_files: usize) -> Self {
        let multi = Arc::new(MultiProgress::new());

        // Create the main progress bar that shows at the bottom
        let main_bar = multi.add(ProgressBar::new(total_files as u64));
        main_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} Downloading {len} files, remaining bytes: {bytes_precise}")
                .unwrap()
                .progress_chars("━━━")  // Unicode box-drawing characters
        );

        Self {
            max_displayed_files: 15,  // Show at most 15 downloaded files
            downloaded_files: Vec::with_capacity(total_files),
            multi,
            main_bar: Arc::new(main_bar),
            current_bar: None,
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
                .template("{bar:40.cyan/blue} {pos}/{len} {bytes_per_sec} {wide_msg}")
                .unwrap()
                .progress_chars("━╾╴")  // Unicode box-drawing characters
        );
        pb.set_message(format!("{}...", download.filename));

        // Update our current bar
        self.current_bar = Some(pb.clone());

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

        // Clear the current download bar
        if let Some(bar) = &self.current_bar {
            bar.finish_and_clear();
        }

        // Redraw the display
        self.redraw();
    }

    /// Redraw the entire display
    fn redraw(&self) {
        // Calculate how many files to show
        let display_count = self.max_displayed_files.min(self.downloaded_files.len());

        // Clear previous output (move cursor up multiple lines and clear)
        if !self.downloaded_files.is_empty() {
            // We need to clear previous lines - the number of displayed files and current download line
            let lines_to_clear = display_count + if self.current_bar.is_some() { 1 } else { 0 };
            for _ in 0..lines_to_clear {
                eprint!("\x1B[1A"); // Move the cursor up one line
                eprint!("\x1B[2K"); // Clear the line
            }
        }

        // Display downloaded files (taking the last N items)
        let start_idx = self.downloaded_files.len().saturating_sub(display_count);
        for (name, completed) in self.downloaded_files.iter().skip(start_idx) {
            if *completed {
                eprintln!("  {}", style(format!("Downloaded {}", name)).green());
            } else {
                eprintln!("  Downloading {}", name);
            }
        }
    }

    /// Set total bytes for the main progress bar
    pub fn set_total_bytes(&self, bytes: u64) {
        self.main_bar.set_length(bytes);
    }

    /// Finish the progress display
    pub fn finish(&mut self) {
        self.main_bar.finish_and_clear();
    }
}
