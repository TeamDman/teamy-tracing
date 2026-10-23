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

#[path = "support/cache_mutation_layer.rs"]
mod cache_mutation_layer;

#[test]
fn warmed_cache_revalidates_external_changes() {
    for replacement in [
        "{\"changed\":18446744073709551615}",
        "{",
        "[]",
        "{\"x\": [\n1]}",
    ] {
        let diagnostics = Diagnostics::default();
        let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
        let subscriber = tracing_subscriber::registry()
            .with(teamy_tracing::layer(writer.clone(), diagnostics.clone()))
            .with(cache_mutation_layer::CacheMutationLayer(replacement));
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("cached", value = 1);
            let _entered = span.enter();
            tracing::info!("warm");
            span.record("value", 2);
            tracing::info!("changed");
        });
        let text = String::from_utf8(writer.inspect(Clone::clone).unwrap()).unwrap();
        let valid = replacement.contains("changed");
        assert_eq!(
            text.lines().count(),
            if valid { 2 } else { 1 },
            "{replacement}"
        );
        assert_eq!(diagnostics.snapshot().formatting_errors, u64::from(!valid));
        if valid {
            assert!(
                text.lines()
                    .last()
                    .unwrap()
                    .contains("18446744073709551615")
            );
            assert!(!text.lines().last().unwrap().contains("\"value\""));
        }
    }
}

#[test]
fn write_zero_stops_delivery_and_counts_once() {
    use std::io::Write;
    use tracing_subscriber::fmt::MakeWriter;
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(&mut [][..], diagnostics.clone());
    {
        let mut guard = writer.make_writer();
        assert_eq!(
            guard.write_all(b"record\n").unwrap_err().kind(),
            std::io::ErrorKind::WriteZero
        );
        assert!(guard.write_all(b"again\n").is_err());
    }
    assert!(writer.make_writer().write_all(b"later\n").is_err());
    assert_eq!(diagnostics.snapshot().write_errors, 1);
    assert_eq!(diagnostics.snapshot().suppressed_records, 1);
}

#[test]
fn poisoned_sink_suppresses_records_without_recovering() {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let panic = std::panic::catch_unwind(|| {
        let _ = writer.inspect(|_| panic!("synthetic panic while lock held"));
    });
    assert!(panic.is_err());
    let subscriber = tracing_subscriber::registry()
        .with(teamy_tracing::layer(writer.clone(), diagnostics.clone()));
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!("suppressed one");
        tracing::info!("suppressed two");
    });
    assert!(writer.inspect(Vec::len).is_err());
    assert!(writer.flush().is_err());
    assert_eq!(diagnostics.snapshot().suppressed_records, 2);
}

#[test]
fn concurrent_recording_and_cache_reads_remain_valid_and_observe_final_update() {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let dispatch = tracing::Dispatch::new(
        tracing_subscriber::registry()
            .with(teamy_tracing::layer(writer.clone(), diagnostics.clone())),
    );
    tracing::dispatcher::with_default(&dispatch, || {
        let span = tracing::info_span!("shared", count = 0u64, exact = u64::MAX);
        std::thread::scope(|scope| {
            for worker in 0..4 {
                let span = span.clone();
                let dispatch = dispatch.clone();
                scope.spawn(move || {
                    tracing::dispatcher::with_default(&dispatch, || {
                        let _entered = span.enter();
                        for count in 0..100u64 {
                            if worker == 0 {
                                span.record("count", count);
                            }
                            tracing::info!(worker);
                        }
                    })
                });
            }
        });
        span.record("count", u64::MAX);
        tracing::info!(parent: &span, "final");
    });
    let text = String::from_utf8(writer.inspect(Clone::clone).unwrap()).unwrap();
    assert_eq!(text.lines().count(), 401);
    for line in text.lines() {
        let _: facet_value::Value = facet_json::from_str(line).unwrap();
        assert!(line.contains("\"exact\":18446744073709551615"));
    }
    assert!(
        text.lines()
            .last()
            .unwrap()
            .contains("\"count\":18446744073709551615")
    );
    assert_eq!(diagnostics.snapshot(), Default::default());
}

#[test]
fn direct_zero_write_and_guard_flush_failure_stop_the_same_request() {
    use std::io::Write;
    use tracing_subscriber::fmt::MakeWriter;
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(&mut [][..], diagnostics.clone());
    let mut guard = writer.make_writer();
    assert_eq!(guard.write(b"").unwrap(), 0);
    assert_eq!(
        guard.write(b"x").unwrap_err().kind(),
        std::io::ErrorKind::WriteZero
    );
    assert!(guard.write(b"y").is_err());
    assert_eq!(diagnostics.snapshot().write_errors, 1);
    assert_eq!(diagnostics.snapshot().suppressed_records, 0);
    drop(guard);

    let writer = SharedWriter::new(
        PartialWriter {
            fail_flush: true,
            ..Default::default()
        },
        diagnostics.clone(),
    );
    let mut guard = writer.make_writer();
    assert!(guard.flush().is_err());
    assert!(guard.write_all(b"same request").is_err());
    assert_eq!(diagnostics.snapshot().flush_errors, 1);
    assert_eq!(diagnostics.snapshot().suppressed_records, 0);
    drop(guard);
    assert!(writer.make_writer().write_all(b"later request").is_err());
    assert_eq!(diagnostics.snapshot().suppressed_records, 1);
}

#[test]
fn file_append_flush_and_reopen_preserve_ndjson_records() {
    use std::fs::OpenOptions;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "teamy-tracing-{}-{nonce}.ndjson",
        std::process::id()
    ));
    let file = OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(&path)
        .unwrap();
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(file, diagnostics.clone());
    for message in ["first", "second"] {
        tracing::subscriber::with_default(
            tracing_subscriber::registry()
                .with(teamy_tracing::layer(writer.clone(), diagnostics.clone())),
            || tracing::info!(message),
        );
        writer.flush().unwrap();
    }
    drop(writer);
    let writer = SharedWriter::new(
        OpenOptions::new().append(true).open(&path).unwrap(),
        diagnostics.clone(),
    );
    tracing::subscriber::with_default(
        tracing_subscriber::registry()
            .with(teamy_tracing::layer(writer.clone(), diagnostics.clone())),
        || tracing::info!("reopened"),
    );
    writer.flush().unwrap();
    drop(writer);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(text.lines().count(), 3);
    assert!(text.ends_with('\n'));
    for line in text.lines() {
        let _: facet_value::Value = facet_json::from_str(line).unwrap();
    }
    assert_eq!(diagnostics.snapshot(), Default::default());
}

#[test]
fn enabled_sinks_keep_filtered_span_scopes_independent() {
    let diagnostics = Diagnostics::default();
    let info = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let debug = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let subscriber = tracing_subscriber::registry()
        .with(
            teamy_tracing::layer(info.clone(), diagnostics.clone())
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .with(
            teamy_tracing::layer(debug.clone(), diagnostics.clone())
                .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG),
        );
    tracing::subscriber::with_default(subscriber, || {
        let parent = tracing::info_span!("visible", value = 0);
        let _parent = parent.enter();
        let child = tracing::debug_span!("debug_context", value = 0);
        let _child = child.enter();
        tracing::info!("warm");
        parent.record("value", 1);
        child.record("value", 2);
        tracing::debug!("debug only");
        tracing::info!("both");
    });
    let info = String::from_utf8(info.inspect(Clone::clone).unwrap()).unwrap();
    let debug = String::from_utf8(debug.inspect(Clone::clone).unwrap()).unwrap();
    assert_eq!(info.lines().count(), 2);
    assert_eq!(debug.lines().count(), 3);
    assert!(!info.contains("debug_context"));
    assert!(!info.contains("debug only"));
    assert!(info.lines().last().unwrap().contains("\"value\":1"));
    assert!(debug.lines().last().unwrap().contains("\"value\":2"));
    assert_eq!(diagnostics.snapshot(), Default::default());
}
