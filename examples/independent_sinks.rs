use teamy_tracing::Diagnostics;
use teamy_tracing::SharedWriter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Substitute a caller-opened append-mode File in an application.
    let diagnostics = Diagnostics::default();
    let file = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let mut file_layer = teamy_tracing::layer(file.clone(), diagnostics.clone());
    file_layer.set_span_events(FmtSpan::CLOSE);
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(tracing_subscriber::EnvFilter::new("warn")),
        )
        .with(file_layer.with_filter(tracing_subscriber::EnvFilter::new("debug")));
    tracing::subscriber::with_default(subscriber, || {
        let _span = tracing::debug_span!("operation", items = 3).entered();
        tracing::debug!(completed = 3, "operation complete");
    });
    file.flush()?;
    assert_eq!(diagnostics.snapshot(), Default::default());
    Ok(())
}
