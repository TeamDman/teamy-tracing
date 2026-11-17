// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
//! Live report-aware tracing capture. Callers own buffers, routing, filters and
//! subscriber installation. This is separate from serializable telemetry: a live
//! eyre report retains typed sources, handlers and captured span context.
mod capture_layer;
mod captured_fields;
mod escape_terminal_controls;
mod format_record;
mod log_buffer;
mod log_buffer_state;
mod log_level;
mod log_record;
mod log_span;
mod log_value;
mod weak_log_buffer;

pub use capture_layer::CaptureLayer;
pub use escape_terminal_controls::escape_terminal_controls;
pub use format_record::format_record;
pub use format_record::format_record_plain;
pub use log_buffer::LogBuffer;
pub use log_level::LogLevel;
pub use log_record::LogRecord;
pub use log_span::LogSpan;
pub use log_value::LogValue;
