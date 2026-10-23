use super::payload_visitor::PayloadVisitor;
use super::startup_fields::StartupFields;
use super::startup_record::StartupRecord;
use teamy_tracing::FacetJson;
use teamy_tracing::FacetJsonFields;
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::fmt::FormatEvent;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::registry::LookupSpan;

#[derive(Debug, Default)]
pub struct StartupFormatter {
    pub fallback: FacetJson,
}
impl<S> FormatEvent<S, FacetJsonFields> for StartupFormatter
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
        let mut visitor = PayloadVisitor::default();
        event.record(&mut visitor);
        // Decode to a known type; producer strings never become unchecked raw JSON.
        let fields: StartupFields =
            facet_json::from_str(visitor.encoded.as_deref().ok_or(std::fmt::Error)?)
                .map_err(|_| std::fmt::Error)?;
        let record = StartupRecord {
            level: event.metadata().level().as_str().into(),
            fields,
            target: event.metadata().target().into(),
        };
        let encoded = facet_json::to_string(&record).map_err(|_| std::fmt::Error)?;
        writeln!(writer, "{encoded}")
    }
}
