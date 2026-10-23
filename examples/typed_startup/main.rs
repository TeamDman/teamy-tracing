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
    let subscriber = tracing_subscriber::registry().with(
        teamy_tracing::layer(writer.clone(), diagnostics).event_format(StartupFormatter::default()),
    );
    let encoded = facet_json::to_string(&StartupFields {
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
    Ok(())
}
