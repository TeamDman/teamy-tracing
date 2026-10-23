use crate::DiagnosticSnapshot;
use crate::diagnostic_counters::DiagnosticCounters;
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// Shared counters without recursion, raw payload retention, or global state.
#[derive(Debug, Default, Clone)]
pub struct Diagnostics(pub(crate) Arc<DiagnosticCounters>);

impl Diagnostics {
    pub fn snapshot(&self) -> DiagnosticSnapshot {
        DiagnosticSnapshot {
            formatting_errors: self.0.formatting_errors.load(Ordering::Relaxed),
            write_errors: self.0.write_errors.load(Ordering::Relaxed),
            flush_errors: self.0.flush_errors.load(Ordering::Relaxed),
            suppressed_records: self.0.suppressed_records.load(Ordering::Relaxed),
        }
    }

    pub(crate) fn format_error(&self) {
        self.0.formatting_errors.fetch_add(1, Ordering::Relaxed);
    }
}
