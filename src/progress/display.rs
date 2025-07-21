//! Progress bar display management and coordination.
//!
//! This module provides the [`ProgressDisplay`] struct that manages and coordinates
//! multiple progress bars during download operations. It handles both the main
//! progress bar (overall download progress) and individual file progress bars.
//!
//! # Examples
//!
//! ## Creating a Progress Display Manager
//!
//! ```rust
//! use trauma::progress::{ProgressDisplay, StyleOptions};
//!
//! let style_options = StyleOptions::default();
//! let total_downloads = 5;
//! let single_file_progress = false;
//!
//! let progress_display = ProgressDisplay::new(
//!     style_options,
//!     total_downloads,
//!     single_file_progress,
//! );
//! ```
//!
//! ## Working with Individual File Progress
//!
//! ```rust,no_run
//! use trauma::progress::{ProgressDisplay, StyleOptions};
//!
//! # async fn example() {
//! let progress_display = ProgressDisplay::new(StyleOptions::default(), 3, false);
//!
//! // Create a progress bar for an individual file
//! let file_progress = progress_display.create_child_progress(1024, 0);
//!
//! // Update progress
//! file_progress.set_position(512);
//! file_progress.set_message("Downloading file.zip...");
//!
//! // Complete the file download
//! progress_display.finish_child(file_progress);
//! # }
//! ```

use crate::progress::StyleOptions;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget};
use std::sync::Arc;

/// Progress display manager that coordinates multiple progress bars.
pub struct ProgressDisplay {
    /// The multi-progress instance for coordinating multiple progress bars.
    multi: Arc<MultiProgress>,
    /// The main progress bar for overall progress.
    main: Arc<ProgressBar>,
    /// Style options for progress bars.
    style_options: StyleOptions,
    /// Whether to show the main progress bar.
    show_main_progress: bool,
}

impl ProgressDisplay {
    /// Create a new progress display manager.
    ///
    /// # Arguments
    /// * `style_options` - Style configuration for progress bars
    /// * `total_downloads` - Total number of downloads for the main progress bar
    /// * `single_file_progress` - Whether to hide main progress for single file downloads
    pub fn new(
        style_options: StyleOptions,
        total_downloads: usize,
        single_file_progress: bool,
    ) -> Self {
        // Prepare the progress bar.
        let multi = match style_options.is_enabled() {
            true => Arc::new(MultiProgress::new()),
            false => Arc::new(MultiProgress::with_draw_target(ProgressDrawTarget::hidden())),
        };

        // Determine if we should show the main progress bar
        let show_main_progress = !single_file_progress || total_downloads > 1;

        let main = if show_main_progress {
            Arc::new(
                multi.add(
                    style_options
                        .main()
                        .clone()
                        .to_progress_bar(total_downloads as u64),
                ),
            )
        } else {
            // Create a completely hidden progress bar that's not added to MultiProgress
            Arc::new(ProgressBar::hidden())
        };

        if show_main_progress {
            main.tick();
        }

        Self {
            multi,
            main,
            style_options,
            show_main_progress,
        }
    }

    /// Get the multi-progress instance for adding child progress bars.
    pub fn multi(&self) -> Arc<MultiProgress> {
        self.multi.clone()
    }

    /// Get the main progress bar.
    pub fn main(&self) -> Arc<ProgressBar> {
        self.main.clone()
    }

    /// Create a child progress bar for individual file downloads.
    ///
    /// # Arguments
    /// * `size` - Total size for the progress bar
    /// * `position` - Starting position (for resume functionality)
    pub fn create_child_progress(&self, size: u64, position: u64) -> ProgressBar {
        self.multi.add(
            self.style_options
                .child()
                .clone()
                .to_progress_bar(size)
                .with_position(position),
        )
    }

    /// Increment the main progress bar by one.
    pub fn increment_main(&self) {
        self.main.inc(1);
    }

    /// Finish the progress display, clearing or keeping bars based on configuration.
    pub fn finish(self) {
        if self.show_main_progress {
            if self.style_options.main().clear {
                self.main.finish_and_clear();
            } else {
                self.main.finish();
            }
        }
    }

    /// Finish a child progress bar based on configuration.
    pub fn finish_child(&self, pb: ProgressBar) {
        if self.style_options.child().clear {
            pb.finish_and_clear();
        } else {
            pb.finish();
        }
    }
}
