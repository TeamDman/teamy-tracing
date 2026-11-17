//! Views over application-owned live capture and retained diagnostics.
//!
//! These widgets do not install a subscriber, own a collector or open a window.
//! The application chooses retention, filtering at capture, repaint scheduling
//! and where each view appears.
#[cfg(feature = "eyre")]
mod diagnostic_report_view;
#[cfg(feature = "eyre")]
mod diagnostic_text_style;
mod escape_display;
mod log_view;

#[cfg(feature = "eyre")]
pub use diagnostic_report_view::DiagnosticReportView;
pub use escape_display::escape_display;
pub use log_view::LogView;
