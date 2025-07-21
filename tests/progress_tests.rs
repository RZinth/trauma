//! Tests for the progress module functionality.
//!
//! This file contains tests for progress bar styling, configuration,
//! and display management functionality.

use trauma::progress::{ProgressBarOpts, ProgressDisplay, StyleOptions};

mod common;
use common::helpers::*;

// Tests moved from src/progress/style.rs
#[test]
fn test_style_options_default() {
    let style = StyleOptions::default();
    assert_style_options_enabled(&style);
    // Test that we can access main and child options
    let _main = style.main();
    let _child = style.child();
}

#[test]
fn test_style_options_new() {
    let style = create_test_style_options();
    assert_style_options_enabled(&style);
    // Test that we can access main and child options
    let _main = style.main();
    let _child = style.child();
}

#[test]
fn test_style_options_disabled() {
    let style = create_disabled_style_options();
    assert_style_options_disabled(&style);
}

#[test]
fn test_style_options_setters() {
    let mut style = StyleOptions::default();
    let new_main = create_hidden_progress_opts();
    let new_child = create_pip_style_progress_opts();

    style.set_main(new_main);
    style.set_child(new_child);

    // Test that the style is still enabled (child is enabled)
    assert_style_options_enabled(&style);
}

#[test]
fn test_progress_bar_opts_default() {
    let opts = create_test_progress_opts();
    assert_progress_opts_enabled(&opts);
    let pb = opts.to_progress_bar(100);
    assert_eq!(pb.length(), Some(100));
}

#[test]
fn test_progress_bar_opts_new() {
    let opts = create_custom_progress_opts("test template", "abc");
    assert_progress_opts_enabled(&opts);
}

#[test]
fn test_progress_bar_opts_hidden() {
    let opts = create_hidden_progress_opts();
    assert_progress_opts_disabled(&opts);
}

#[test]
fn test_progress_bar_opts_with_pip_style() {
    let opts = create_pip_style_progress_opts();
    assert_progress_opts_enabled(&opts);
    let pb = opts.to_progress_bar(100);
    assert_eq!(pb.length(), Some(100));
}

#[test]
fn test_progress_bar_opts_set_clear() {
    let mut opts = ProgressBarOpts::default();
    
    // Test that set_clear doesn't break the progress bar creation
    opts.set_clear(false);
    let pb = opts.clone().to_progress_bar(100);
    assert!(!pb.is_hidden());
    
    opts.set_clear(true);
    let pb2 = opts.to_progress_bar(100);
    assert!(!pb2.is_hidden());
}

#[test]
fn test_progress_bar_opts_to_progress_bar_hidden() {
    let opts = ProgressBarOpts::hidden();
    let pb = opts.to_progress_bar(100);
    assert!(pb.is_hidden());
}

#[test]
fn test_progress_bar_opts_to_progress_bar_enabled() {
    let opts = ProgressBarOpts::default();
    let pb = opts.to_progress_bar(100);
    assert!(!pb.is_hidden());
    assert_eq!(pb.length(), Some(100));
}

// Tests moved from src/progress/display.rs
#[test]
fn test_progress_display_new_single_file() {
    let style = create_test_style_options();
    let display = ProgressDisplay::new(style, 1, true);

    // Test that the display was created successfully
    let _multi = display.multi();
    let main = display.main();
    // For single file with single_file_progress=true, main progress bar is hidden
    assert!(main.is_hidden() || main.length() == Some(1));
}

#[test]
fn test_progress_display_new_multiple_files() {
    let style = create_test_style_options();
    let display = ProgressDisplay::new(style, 3, true);

    // Test that the display was created successfully
    let _multi = display.multi();
    let main = display.main();
    assert_eq!(main.length(), Some(3));
}

#[test]
fn test_progress_display_new_single_file_progress_disabled() {
    let style = create_test_style_options();
    let display = ProgressDisplay::new(style, 1, false);

    // Test that the display was created successfully
    let _multi = display.multi();
    let main = display.main();
    assert_eq!(main.length(), Some(1));
}

#[test]
fn test_progress_display_new_disabled_style() {
    let style = create_disabled_style_options();
    let display = ProgressDisplay::new(style, 3, false);

    // Test that the display was created successfully even with disabled style
    let _multi = display.multi();
    let main = display.main();
    // With disabled style, the main progress bar should still be created but may be hidden
    assert!(main.is_hidden() || main.length() == Some(3));
}

#[test]
fn test_progress_display_create_child_progress() {
    let style = create_test_style_options();
    let display = ProgressDisplay::new(style, 1, false);

    let child_pb = display.create_child_progress(1000, 500);
    assert_eq!(child_pb.length(), Some(1000));
    assert_eq!(child_pb.position(), 500);
}

#[test]
fn test_progress_display_increment_main() {
    let style = create_test_style_options();
    let display = ProgressDisplay::new(style, 3, false);

    let initial_position = display.main().position();
    display.increment_main();
    assert_eq!(display.main().position(), initial_position + 1);
}

#[test]
fn test_progress_display_finish_child() {
    let style = create_test_style_options();
    let display = ProgressDisplay::new(style, 1, false);

    let child_pb = display.create_child_progress(100, 0);
    child_pb.set_position(100);

    // This should not panic and should handle the finish properly
    display.finish_child(child_pb);
}

#[test]
fn test_progress_display_multi_and_main_access() {
    let style = create_test_style_options();
    let display = ProgressDisplay::new(style, 2, false);

    let _multi = display.multi();
    let main = display.main();

    // Should be able to access both multi and main progress bars
    assert_eq!(main.length(), Some(2));
}