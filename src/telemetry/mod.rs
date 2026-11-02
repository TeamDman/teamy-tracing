//! Bounded, in-memory records for external and local telemetry.
//!
//! This module does not install subscribers, start listeners or write files.
mod attribute;
mod correlation_key;
mod record;
mod record_store;
mod store_limits;
mod store_stats;
mod value;

#[cfg(feature = "otlp")]
pub mod otlp;
#[cfg(feature = "otlp-http")]
pub mod otlp_http;
#[cfg(feature = "egui")]
pub mod view;

pub use attribute::Attribute;
pub use correlation_key::CorrelationKey;
pub use record::TelemetryRecord;
pub use record_store::RecordStore;
pub use store_limits::StoreLimits;
pub use store_stats::StoreStats;
pub use value::TelemetryValue;
