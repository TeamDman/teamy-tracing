use std::sync::atomic::AtomicU64;

#[derive(Debug, Default)]
pub(crate) struct DiagnosticCounters {
    pub formatting_errors: AtomicU64,
    pub write_errors: AtomicU64,
    pub flush_errors: AtomicU64,
    pub suppressed_records: AtomicU64,
}
