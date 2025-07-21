//! Progress bar styling and configuration options.
//!
//! This module provides styling and configuration options for progress bars
//! used during download operations. It supports both main progress bars
//! (showing overall progress) and child progress bars (showing individual file progress).
//!
//! # Examples
//!
//! ## Default Styling
//!
//! ```rust
//! use trauma::progress::StyleOptions;
//!
//! // Use default styling (main bar stays visible, child bars clear on completion)
//! let style_options = StyleOptions::default();
//! ```
//!
//! ## Custom Styling
//!
//! ```rust
//! use trauma::progress::{StyleOptions, ProgressBarOpts};
//!
//! let custom_style = StyleOptions::new(
//!     ProgressBarOpts::new(
//!         Some("[{bar:40.cyan/blue}] {pos}/{len} {msg}".to_string()),
//!         Some("█▉▊▋▌▍▎▏  ".to_string()),
//!         true,
//!         false
//!     ),
//!     ProgressBarOpts::with_pip_style(),
//! );
//! ```
//!
//! ## Hidden Progress Bars
//!
//! ```rust
//! use trauma::progress::{StyleOptions, ProgressBarOpts};
//!
//! let hidden_style = StyleOptions::new(
//!     ProgressBarOpts::hidden(),
//!     ProgressBarOpts::hidden(),
//! );
//! ```
//!
//! ## Checking if Progress is Enabled
//!
//! ```rust
//! use trauma::progress::StyleOptions;
//!
//! let style_options = StyleOptions::default();
//! if style_options.is_enabled() {
//!     println!("Progress bars are enabled");
//! }
//! ```

use indicatif::{ProgressBar, ProgressStyle};

/// Define the downloader style options.
///
/// By default, the main progress bar will stay on the screen upon completion,
/// but the child ones will be cleared once complete.
#[derive(Debug, Clone)]
pub struct StyleOptions {
    /// Style options for the main progress bar.
    pub(crate) main: ProgressBarOpts,
    /// Style options for the child progress bar(s).
    pub(crate) child: ProgressBarOpts,
}

impl Default for StyleOptions {
    fn default() -> Self {
        Self {
            main: ProgressBarOpts {
                template: Some(ProgressBarOpts::TEMPLATE_BAR_WITH_POSITION.into()),
                progress_chars: Some(ProgressBarOpts::CHARS_FINE.into()),
                enabled: true,
                clear: false,
            },
            child: ProgressBarOpts::with_pip_style(),
        }
    }
}

impl StyleOptions {
    /// Create new [`StyleOptions`].
    pub fn new(main: ProgressBarOpts, child: ProgressBarOpts) -> Self {
        Self { main, child }
    }

    /// Set the options for the main progress bar.
    pub fn set_main(&mut self, main: ProgressBarOpts) {
        self.main = main;
    }

    /// Set the options for the child progress bar.
    pub fn set_child(&mut self, child: ProgressBarOpts) {
        self.child = child;
    }

    /// Return `false` if neither the main nor the child bar is enabled.
    pub fn is_enabled(&self) -> bool {
        self.main.enabled || self.child.enabled
    }

    /// Get a reference to the main progress bar options.
    pub fn main(&self) -> &ProgressBarOpts {
        &self.main
    }

    /// Get a reference to the child progress bar options.
    pub fn child(&self) -> &ProgressBarOpts {
        &self.child
    }
}

/// Define the options for a progress bar.
#[derive(Debug, Clone)]
pub struct ProgressBarOpts {
    /// Progress bar template string.
    template: Option<String>,
    /// Progression characters set.
    ///
    /// There must be at least 3 characters for the following states:
    /// "filled", "current", and "to do".
    progress_chars: Option<String>,
    /// Enable or disable the progress bar.
    pub(crate) enabled: bool,
    /// Clear the progress bar once completed.
    pub(crate) clear: bool,
}

impl Default for ProgressBarOpts {
    fn default() -> Self {
        Self {
            template: None,
            progress_chars: None,
            enabled: true,
            clear: true,
        }
    }
}

impl ProgressBarOpts {
    /// Template representing the bar and its position.
    ///
    ///`███████████████████████████████████████ 11/12 (99%) eta 00:00:02`
    pub const TEMPLATE_BAR_WITH_POSITION: &'static str =
        "{bar:40.blue} {pos:>}/{len} ({percent}%) eta {eta_precise:.blue}";
    /// Template which looks like the Python package installer pip.
    ///
    /// `━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 211.23 KiB/211.23 KiB 1008.31 KiB/s eta 0s`
    pub const TEMPLATE_PIP: &'static str =
        "{bar:40.green/black} {bytes:>11.green}/{total_bytes:<11.green} {bytes_per_sec:>13.red} eta {eta:.blue}";
    /// Use increasing quarter blocks as progress characters: `"█▛▌▖  "`.
    pub const CHARS_BLOCKY: &'static str = "█▛▌▖  ";
    /// Use fade-in blocks as progress characters: `"█▓▒░  "`.
    pub const CHARS_FADE_IN: &'static str = "█▓▒░  ";
    /// Use fine blocks as progress characters: `"█▉▊▋▌▍▎▏  "`.
    pub const CHARS_FINE: &'static str = "█▉▊▋▌▍▎▏  ";
    /// Use a line as progress characters: `"━╾─"`.
    pub const CHARS_LINE: &'static str = "━╾╴─";
    /// Use rough blocks as progress characters: `"█  "`.
    pub const CHARS_ROUGH: &'static str = "█  ";
    /// Use increasing height blocks as progress characters: `"█▇▆▅▄▃▂   "`.
    pub const CHARS_VERTICAL: &'static str = "█▇▆▅▄▃▂   ";

    /// Create a new [`ProgressBarOpts`].
    pub fn new(
        template: Option<String>,
        progress_chars: Option<String>,
        enabled: bool,
        clear: bool,
    ) -> Self {
        Self {
            template,
            progress_chars,
            enabled,
            clear,
        }
    }

    /// Create a [`ProgressStyle`] based on the provided options.
    pub fn to_progress_style(self) -> ProgressStyle {
        let mut style = ProgressStyle::default_bar();
        if let Some(template) = self.template {
            style = style.template(&template).unwrap();
        }
        if let Some(progress_chars) = self.progress_chars {
            style = style.progress_chars(&progress_chars);
        }
        style
    }

    /// Create a [`ProgressBar`] based on the provided options.
    pub fn to_progress_bar(self, len: u64) -> ProgressBar {
        // Return a hidden Progress bar if we disabled it.
        if !self.enabled {
            return ProgressBar::hidden();
        }

        // Otherwise returns a ProgressBar with the style.
        let style = self.to_progress_style();
        ProgressBar::new(len).with_style(style)
    }

    /// Create a new [`ProgressBarOpts`] which looks like Python pip.
    pub fn with_pip_style() -> Self {
        Self {
            template: Some(ProgressBarOpts::TEMPLATE_PIP.into()),
            progress_chars: Some(ProgressBarOpts::CHARS_LINE.into()),
            enabled: true,
            clear: true,
        }
    }

    /// Set to `true` to clear the progress bar upon completion.
    pub fn set_clear(&mut self, clear: bool) {
        self.clear = clear;
    }

    /// Create a new [`ProgressBarOpts`] which hides the progress bars.
    pub fn hidden() -> Self {
        Self {
            enabled: false,
            ..ProgressBarOpts::default()
        }
    }
}
