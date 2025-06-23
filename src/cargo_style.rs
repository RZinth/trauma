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

        let main_bar = multi.add(ProgressBar::new(total_files as u64));
        main_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} Downloading {pos}/{len} files")
                .unwrap()
        );

        Self {
            max_displayed_files: 10,
            downloaded_files: Vec::with_capacity(total_files),
            multi,
            main_bar: Arc::new(main_bar),
            current_bar: None,
        }
    }

    /// Adds a new download to the list
    pub fn add_download(&mut self, download: &Download) -> ProgressBar {
        self.downloaded_files.push((download.filename.clone(), false));

        let pb = self.multi.add(ProgressBar::new(0));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.cyan/blue} {pos}/{len} {bytes_per_sec} {wide_msg}")
                .unwrap()
                .progress_chars("━╾╴")
        );
        pb.set_message(format!("{}...", download.filename));

        self.current_bar = Some(pb.clone());

        self.redraw();

        pb
    }

    /// Mark a download as completed
    pub fn complete_download(&mut self, filename: &str) {
        if let Some(idx) = self.downloaded_files.iter().position(|(name, _)| name == filename) {
            self.downloaded_files[idx].1 = true;
        }

        self.main_bar.inc(1);

        if let Some(bar) = &self.current_bar {
            bar.finish_and_clear();
        }

        self.redraw();
    }

    /// Redraw the entire display
    fn redraw(&self) {
        let display_count = self.max_displayed_files.min(self.downloaded_files.len());

        if !self.downloaded_files.is_empty() {
            let lines_to_clear = display_count + if self.current_bar.is_some() { 1 } else { 0 };
            for _ in 0..lines_to_clear {
                eprint!("\x1B[1A");
                eprint!("\x1B[2K");
            }
        }

        let start_idx = self.downloaded_files.len().saturating_sub(display_count);
        for (name, completed) in self.downloaded_files.iter().skip(start_idx) {
            if *completed {
                eprintln!("  {}", style(format!("Downloaded {}", name)).green());
            } 
            else { 
                // ...
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