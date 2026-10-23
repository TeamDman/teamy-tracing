use super::payload_visitor::PayloadVisitor;
use super::startup_fields::StartupFields;
use super::startup_record::StartupRecord;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use teamy_tracing::Diagnostics;
use teamy_tracing::FacetJson;
use teamy_tracing::FacetJsonFields;
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::fmt::FormatEvent;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::time::SystemTime;
use tracing_subscriber::registry::LookupSpan;

/// Proposed startup adapter, not a copy of an unavailable consumer formatter.
/// Ordinary events retain the library's counters. Reserved startup failures have
/// a separate counter because Diagnostics intentionally exposes no mutation API.
#[derive(Debug, Clone)]
pub struct StartupFormatter<T = SystemTime> {
    fallback: FacetJson<T>,
    timer: T,
    errors: Arc<AtomicU64>,
}
impl StartupFormatter {
    pub fn new(diagnostics: Diagnostics) -> Self {
        Self {
            fallback: FacetJson::default()
                .with_file(true)
                .with_line_number(true)
                .with_diagnostics(diagnostics),
            timer: SystemTime,
            errors: Arc::default(),
        }
    }
}
impl<T> StartupFormatter<T> {
    pub fn with_timer<U: Clone>(self, timer: U) -> StartupFormatter<U> {
        StartupFormatter {
            fallback: self.fallback.with_timer(timer.clone()),
            timer,
            errors: self.errors,
        }
    }
    pub fn formatting_errors(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }
}
impl<T: FormatTime> StartupFormatter<T> {
    fn encode(&self, event: &tracing::Event<'_>) -> Result<String, std::fmt::Error> {
        let mut visitor = PayloadVisitor::default();
        event.record(&mut visitor);
        // Decode and reserialize the known schema; no untrusted raw JSON splice.
        let fields: StartupFields =
            facet_json::from_str(visitor.encoded.as_deref().ok_or(std::fmt::Error)?)
                .map_err(|_| std::fmt::Error)?;
        let mut timestamp = String::new();
        self.timer.format_time(&mut Writer::new(&mut timestamp))?;
        let metadata = event.metadata();
        let record = StartupRecord {
            timestamp,
            level: metadata.level().as_str().into(),
            fields,
            target: metadata.target().into(),
            filename: metadata.file().map(String::from),
            line_number: metadata.line(),
        };
        facet_json::to_string(&record).map_err(|_| std::fmt::Error)
    }
}
impl<S, T: FormatTime> FormatEvent<S, FacetJsonFields> for StartupFormatter<T>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, FacetJsonFields>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        if event.metadata().target() != "example::startup" {
            return self.fallback.format_event(ctx, writer, event);
        }
        let result = self
            .encode(event)
            .and_then(|encoded| writeln!(writer, "{encoded}"));
        if result.is_err() {
            self.errors.fetch_add(1, Ordering::Relaxed);
        }
        result
    }
}
