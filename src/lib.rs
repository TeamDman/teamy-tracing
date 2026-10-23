//! Facet JSON formatting with upstream subscriber registry, filtering and layers.
//!
//! Supply writers and install the subscriber at the application boundary. The
//! library never acquires stdout/stderr or installs a global subscriber.
//!
//! # Example
//!
//! ```
//! use teamy_tracing::{Diagnostics, SharedWriter};
//! use tracing_subscriber::prelude::*;
//!
//! let diagnostics = Diagnostics::default();
//! let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
//! let subscriber = tracing_subscriber::registry()
//!     .with(teamy_tracing::layer(writer.clone(), diagnostics.clone()));
//! tracing::subscriber::with_default(subscriber, || {
//!     let span = tracing::info_span!("operation", completed = 0u64);
//!     let _entered = span.enter();
//!     span.record("completed", 3u64);
//!     tracing::info!("finished");
//! });
//! writer.flush().unwrap();
//! assert_eq!(diagnostics.snapshot(), Default::default());
//! ```
//!
//! [`FacetJson`] controls JSON settings; replacing upstream's formatter does not
//! inherit upstream `Format` options. [`layer`] disables ANSI and plain-text
//! internal-error fallback. Apply filters independently to each layer.
//!
//! [`SharedWriter`] serializes whole-record writes and stops delivery after an
//! I/O error. A failed write may leave a truncated last line. Explicit flush is
//! fallible and is not fsync. Writer callbacks must not recurse into tracing,
//! `inspect`, or `flush`. Inspect [`Diagnostics`] outside logging callbacks.
//!
//! [`DiagnosticSnapshot`] implements the Facet trait from the exact owned
//! registry dependency (`teamy-facet`), not an upstream or Git Facet identity.
//! The public subscriber traits remain those of crates.io `tracing-subscriber`.
//! Log-facade normalization and unstable valuable support are not implemented.
mod diagnostic_counters;
mod diagnostic_snapshot;
mod diagnostics;
mod facet_json;
mod facet_json_fields;
mod hex_bytes;
mod json_options;
mod json_visitor;
mod record_guard;
mod serialize_scalar;
mod shared_writer;
mod validated_span;
mod writer_state;

pub use diagnostic_snapshot::DiagnosticSnapshot;
pub use diagnostics::Diagnostics;
pub use facet_json::FacetJson;
pub use facet_json_fields::FacetJsonFields;
pub use record_guard::RecordGuard;
pub use shared_writer::SharedWriter;

/// Construct a compact JSON layer with no ANSI or plain-text fallback diagnostics.
pub fn layer<S, W>(
    writer: W,
    diagnostics: Diagnostics,
) -> tracing_subscriber::fmt::Layer<S, FacetJsonFields, FacetJson, W>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    W: for<'a> tracing_subscriber::fmt::MakeWriter<'a> + 'static,
{
    tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .log_internal_errors(false)
        .fmt_fields(FacetJsonFields::new(diagnostics.clone()))
        .event_format(FacetJson::default().with_diagnostics(diagnostics))
        .with_writer(writer)
}
