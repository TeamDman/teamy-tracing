//! Facet JSON formatting with upstream subscriber registry, filtering and layers.
//!
//! No global subscriber is installed. Supply a writer at the application boundary.
//! See the README for schema compatibility and delivery failure semantics.
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
