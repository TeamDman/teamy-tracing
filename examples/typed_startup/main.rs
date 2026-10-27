mod payload_visitor;
mod startup_fields;
mod startup_formatter;
mod startup_record;
use startup_fields::StartupFields;
use startup_formatter::StartupFormatter;
use teamy_tracing::Diagnostics;
use teamy_tracing::SharedWriter;
use tracing_subscriber::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let formatter = StartupFormatter::new(diagnostics.clone())
        .with_timer(tracing_subscriber::fmt::time::SystemTime);
    let subscriber = tracing_subscriber::registry().with(
        teamy_tracing::json_layer(writer.clone(), diagnostics.clone())
            .event_format(formatter.clone()),
    );
    let encoded = facet_json::to_string(&StartupFields {
        message: "Application started".into(),
        version: "0.1.0 (repo https://example.invalid/tool, branch fixture, rev synthetic, worktree clean, built 2026-10-23T00:00:00Z)".into(),
        argv: vec!["synthetic".into(), "line\nbreak".into()],
    })?;
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(target: "example::startup", encoded = encoded.as_str());
        tracing::info!("ordinary fallback");
    });
    writer.flush()?;
    let text = String::from_utf8(writer.inspect(Clone::clone)?)?;
    let first = text.lines().next().ok_or("missing startup event")?;
    let record: startup_record::StartupRecord = facet_json::from_str(first)?;
    assert_eq!(record.fields.argv[1], "line\nbreak");
    assert_eq!(text.lines().count(), 2);
    assert_eq!(formatter.formatting_errors(), 0);
    assert_eq!(diagnostics.snapshot(), Default::default());
    Ok(())
}
