mod fixed_time;
pub use fixed_time::FixedTime;
use teamy_tracing::DiagnosticSnapshot;
use teamy_tracing::Diagnostics;
use teamy_tracing::FacetJson;
use teamy_tracing::FacetJsonFields;
use teamy_tracing::SharedWriter;
use tracing_subscriber::prelude::*;

pub fn capture(format: FacetJson<FixedTime>, emit: impl FnOnce()) -> (String, DiagnosticSnapshot) {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .log_internal_errors(false)
            .fmt_fields(FacetJsonFields::new(diagnostics.clone()))
            .event_format(format.with_diagnostics(diagnostics.clone()))
            .with_writer(writer.clone()),
    );
    tracing::subscriber::with_default(subscriber, emit);
    (
        String::from_utf8(writer.inspect(Clone::clone).unwrap()).unwrap(),
        diagnostics.snapshot(),
    )
}

pub fn format() -> FacetJson<FixedTime> {
    FacetJson::default().with_timer(FixedTime)
}

pub fn raw(text: &str) -> std::collections::BTreeMap<String, facet_json::RawJson<'static>> {
    facet_json::from_str(text).unwrap()
}
