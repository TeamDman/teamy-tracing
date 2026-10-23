/// Monotonic, independently sampled counters; inspect outside tracing callbacks.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, facet::Facet)]
pub struct DiagnosticSnapshot {
    pub formatting_errors: u64,
    pub write_errors: u64,
    pub flush_errors: u64,
    pub suppressed_records: u64,
}
