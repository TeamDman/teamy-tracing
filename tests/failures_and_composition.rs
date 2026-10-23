#[path = "support/bad_debug.rs"]
mod bad_debug;
#[path = "support/corrupt_cache_layer.rs"]
mod corrupt_cache_layer;
#[path = "support/failing_time.rs"]
mod failing_time;
#[path = "support/partial_writer.rs"]
mod partial_writer;
use bad_debug::BadDebug;
use failing_time::FailingTime;
use partial_writer::PartialWriter;
use teamy_tracing::Diagnostics;
use teamy_tracing::FacetJson;
use teamy_tracing::SharedWriter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

#[test]
fn concurrent_short_writes_keep_records_whole() {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(PartialWriter::default(), diagnostics.clone());
    let dispatch = tracing::Dispatch::new(
        tracing_subscriber::registry()
            .with(teamy_tracing::layer(writer.clone(), diagnostics.clone())),
    );
    std::thread::scope(|scope| {
        for thread in 0..4 {
            let dispatch = dispatch.clone();
            scope.spawn(move || {
                tracing::dispatcher::with_default(&dispatch, || {
                    for sequence in 0..50 {
                        tracing::info!(thread, sequence, "record\nwith embedded LF");
                    }
                })
            });
        }
    });
    writer.flush().unwrap();
    let bytes = writer.inspect(|w| w.bytes.clone()).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert_eq!(text.lines().count(), 200);
    for line in text.lines() {
        let _: facet_value::Value = facet_json::from_str(line).unwrap();
    }
    assert_eq!(diagnostics.snapshot(), Default::default());
}

#[test]
fn partial_io_failure_stops_later_records_and_is_observable() {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(
        PartialWriter {
            fail_after: Some(7),
            ..Default::default()
        },
        diagnostics.clone(),
    );
    let subscriber = tracing_subscriber::registry()
        .with(teamy_tracing::layer(writer.clone(), diagnostics.clone()));
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!("first");
        tracing::info!("must not append");
    });
    assert_eq!(writer.inspect(|w| w.bytes.len()).unwrap(), 7);
    assert!(writer.flush().is_err());
    let status = diagnostics.snapshot();
    assert_eq!(status.write_errors, 1);
    assert_eq!(status.suppressed_records, 1);
}

#[test]
fn flush_failure_is_explicit_and_stops_delivery() {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(
        PartialWriter {
            fail_flush: true,
            ..Default::default()
        },
        diagnostics.clone(),
    );
    assert!(writer.flush().is_err());
    let subscriber = tracing_subscriber::registry()
        .with(teamy_tracing::layer(writer.clone(), diagnostics.clone()));
    tracing::subscriber::with_default(subscriber, || tracing::info!("suppressed"));
    assert_eq!(diagnostics.snapshot().flush_errors, 1);
    assert_eq!(diagnostics.snapshot().suppressed_records, 1);
}

#[test]
fn timer_failure_drops_entire_event_without_plaintext_fallback() {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let layer = teamy_tracing::layer(writer.clone(), diagnostics.clone()).event_format(
        FacetJson::default()
            .with_timer(FailingTime)
            .with_diagnostics(diagnostics.clone()),
    );
    tracing::subscriber::with_default(tracing_subscriber::registry().with(layer), || {
        tracing::info!(secret = "not emitted")
    });
    assert!(writer.inspect(Vec::is_empty).unwrap());
    assert_eq!(diagnostics.snapshot().formatting_errors, 1);
}

#[test]
fn bad_debug_drops_event_and_keeps_span_cache_valid() {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let subscriber = tracing_subscriber::registry()
        .with(teamy_tracing::layer(writer.clone(), diagnostics.clone()));
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(broken = ?BadDebug);
        let span = tracing::info_span!("bad_span", value = ?BadDebug);
        let _entered = span.enter();
        tracing::info!("valid context");
        span.record("value", 42);
        span.record("value", tracing::field::debug(BadDebug));
        tracing::info!("previous field retained");
    });
    let text = String::from_utf8(writer.inspect(Clone::clone).unwrap()).unwrap();
    assert_eq!(text.lines().count(), 2);
    assert!(text.lines().last().unwrap().contains("\"value\":42"));
    for line in text.lines() {
        let _: facet_value::Value = facet_json::from_str(line).unwrap();
    }
    assert_eq!(diagnostics.snapshot().formatting_errors, 3);
}

#[test]
fn independent_filters_error_layer_and_span_close_compose() {
    let diagnostics = Diagnostics::default();
    let terminal = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let file = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let mut file_layer = teamy_tracing::layer(file.clone(), diagnostics.clone());
    file_layer.set_span_events(FmtSpan::CLOSE);
    let subscriber = tracing_subscriber::registry()
        .with(tracing_error::ErrorLayer::default())
        .with(
            teamy_tracing::layer(terminal.clone(), diagnostics.clone())
                .with_filter(tracing_subscriber::EnvFilter::new("off")),
        )
        .with(file_layer.with_filter(tracing_subscriber::EnvFilter::new("debug")));
    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::debug_span!("work", count = 1);
        let _entered = span.enter();
        tracing::debug!("work done");
    });
    assert!(terminal.inspect(Vec::is_empty).unwrap());
    let text = String::from_utf8(file.inspect(Clone::clone).unwrap()).unwrap();
    assert_eq!(text.lines().count(), 2);
    assert!(text.contains("\"message\":\"close\""));
    assert!(text.contains("\"time.busy\":\""));
    assert!(text.contains("\"time.idle\":\""));
    assert_eq!(diagnostics.snapshot(), Default::default());
}

#[test]
fn corrupt_multiline_cache_drops_event_instead_of_breaking_ndjson() {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let subscriber = tracing_subscriber::registry()
        .with(teamy_tracing::layer(writer.clone(), diagnostics.clone()))
        .with(corrupt_cache_layer::CorruptCacheLayer);
    tracing::subscriber::with_default(subscriber, || {
        let _entered = tracing::info_span!("corrupted").entered();
        tracing::info!("must not emit a multiline record");
    });
    assert!(writer.inspect(Vec::is_empty).unwrap());
    assert_eq!(diagnostics.snapshot().formatting_errors, 1);
}
