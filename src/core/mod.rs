//! Core image processing, capture, and annotation engine for HyperShot.
#![allow(unused_imports)]

pub mod capture;
pub mod filters;
pub mod counter;
pub mod annotator;
pub mod stitching;
pub mod clipboard;
pub mod config;

pub use capture::{CaptureEngine, MonitorInfo};
pub use filters::{pixelate, blur, Rect};
pub use counter::{CounterManager, StepBadge};
pub use annotator::{draw_arrow, draw_rectangle, draw_highlighter, ColorRgba};
pub use stitching::ScrollStitcher;
pub use clipboard::copy_to_clipboard;
pub use config::AppConfig;
