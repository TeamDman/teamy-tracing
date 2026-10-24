#[path = "support/gated_writer.rs"]
mod gated_writer;
#[path = "support/interrupted_writer.rs"]
mod interrupted_writer;
#[path = "support/partial_writer.rs"]
mod partial_writer;
use gated_writer::GatedWriter;
use interrupted_writer::InterruptedWriter;
use partial_writer::PartialWriter;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;
use teamy_tracing::Diagnostics;
use teamy_tracing::SharedWriter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

#[test]
fn cooperative_cancellation_finishes_in_flight_record_closes_span_and_checks_flush() {
    // AtomicBool represents caller-owned cancellation, not teamy-cancellation
    // integration or a promise that this synchronous writer interrupts blocked I/O.
    for fail_flush in [false, true] {
        let diagnostics = Diagnostics::default();
        let (started_tx, started_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        let writer = SharedWriter::new(
            GatedWriter {
                bytes: Vec::new(),
                started: Some(started_tx),
                release: release_rx,
                fail_flush,
            },
            diagnostics.clone(),
        );
        let mut layer = teamy_tracing::layer(writer.clone(), diagnostics.clone());
        layer.set_span_events(FmtSpan::CLOSE);
        let dispatch = tracing::Dispatch::new(tracing_subscriber::registry().with(layer));
        let cancellation = Arc::new(AtomicBool::new(false));
        let worker_cancellation = Arc::clone(&cancellation);
        let worker_dispatch = dispatch.clone();
        let worker = std::thread::spawn(move || {
            tracing::dispatcher::with_default(&worker_dispatch, || {
                let span = tracing::info_span!("work", completed = 0u64);
                let _entered = span.enter();
                let mut completed = 0u64;
                while completed < 10 && !worker_cancellation.load(Ordering::Acquire) {
                    tracing::info!(completed, "record in flight\nwith newline");
                    completed += 1;
                    span.record("completed", completed);
                }
                completed
            })
        });
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        cancellation.store(true, Ordering::Release);
        release_tx.send(()).unwrap();
        assert_eq!(worker.join().unwrap(), 1);
        assert_eq!(writer.flush().is_err(), fail_flush);
        let text = String::from_utf8(writer.inspect(|w| w.bytes.clone()).unwrap()).unwrap();
        assert_eq!(text.lines().count(), 2);
        assert_eq!(text.bytes().filter(|b| *b == b'\n').count(), 2);
        for line in text.lines() {
            let _: facet_value::Value = facet_json::from_str(line).unwrap();
        }
        let close = text.lines().last().unwrap();
        assert!(close.contains("\"message\":\"close\""));
        assert!(close.contains("\"completed\":1"));
        assert!(close.contains("\"time.busy\":"));
        assert_eq!(diagnostics.snapshot().flush_errors, u64::from(fail_flush));
        if fail_flush {
            tracing::dispatcher::with_default(&dispatch, || {
                tracing::info!("must not append after failed shutdown")
            });
            assert_eq!(writer.inspect(|w| w.bytes.len()).unwrap(), text.len());
            assert_eq!(diagnostics.snapshot().suppressed_records, 1);
        } else {
            assert_eq!(diagnostics.snapshot(), Default::default());
        }
    }
}

#[test]
fn one_failed_sink_does_not_stop_an_independently_filtered_healthy_sink() {
    // Exercise both stacking orders; diagnostics belong to each sink.
    for failing_first in [false, true] {
        let failed_status = Diagnostics::default();
        let healthy_status = Diagnostics::default();
        let failed = SharedWriter::new(
            PartialWriter {
                fail_after: Some(7),
                ..Default::default()
            },
            failed_status.clone(),
        );
        let healthy = SharedWriter::new(Vec::<u8>::new(), healthy_status.clone());
        let dispatch = if failing_first {
            tracing::Dispatch::new(
                tracing_subscriber::registry()
                    .with(
                        teamy_tracing::layer(failed.clone(), failed_status.clone())
                            .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
                    )
                    .with(
                        teamy_tracing::layer(healthy.clone(), healthy_status.clone())
                            .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG),
                    ),
            )
        } else {
            tracing::Dispatch::new(
                tracing_subscriber::registry()
                    .with(
                        teamy_tracing::layer(healthy.clone(), healthy_status.clone())
                            .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG),
                    )
                    .with(
                        teamy_tracing::layer(failed.clone(), failed_status.clone())
                            .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
                    ),
            )
        };
        tracing::dispatcher::with_default(&dispatch, || {
            tracing::info!("first");
            tracing::debug!("healthy only");
            tracing::info!("later");
        });
        assert!(failed.flush().is_err());
        healthy.flush().unwrap();
        assert_eq!(failed.inspect(|w| w.bytes.len()).unwrap(), 7);
        let text = String::from_utf8(healthy.inspect(Clone::clone).unwrap()).unwrap();
        assert_eq!(text.lines().count(), 3);
        for line in text.lines() {
            let _: facet_value::Value = facet_json::from_str(line).unwrap();
        }
        assert_eq!(failed_status.snapshot().write_errors, 1);
        assert_eq!(failed_status.snapshot().suppressed_records, 1);
        assert_eq!(healthy_status.snapshot(), Default::default());
    }
}

#[test]
fn interrupted_short_writes_retry_without_poisoning_the_sink() {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(InterruptedWriter::default(), diagnostics.clone());
    tracing::subscriber::with_default(
        tracing_subscriber::registry()
            .with(teamy_tracing::layer(writer.clone(), diagnostics.clone())),
        || {
            tracing::info!("first");
            tracing::info!("second");
        },
    );
    writer.flush().unwrap();
    let text = String::from_utf8(writer.inspect(|w| w.bytes.clone()).unwrap()).unwrap();
    assert_eq!(text.lines().count(), 2);
    for line in text.lines() {
        let _: facet_value::Value = facet_json::from_str(line).unwrap();
    }
    assert!(writer.inspect(|w| w.calls).unwrap() > 2);
    assert_eq!(diagnostics.snapshot(), Default::default());
}
