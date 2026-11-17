// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use super::LogBuffer;
use super::LogLevel;
use super::LogRecord;
use super::LogSpan;
use super::captured_fields::CapturedFields;
use super::weak_log_buffer::WeakLogBuffer;
use std::fmt;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::Subscriber;
use tracing::span::Attributes;
use tracing::span::Id;
use tracing::span::Record;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

/// A report-aware event layer with application-owned routing and storage.
///
/// Capture while `active` returns true; otherwise call the supplied sink. Keep
/// a separate `LogBuffer` alive for capture. The layer owns only a weak writer,
/// preventing retained reports' span traces from owning their own buffer.
/// Once the buffer owner is dropped, active events are discarded. Inactive
/// events still reach the sink. Apply per-layer filters with `Layer::with_filter`.
///
/// Neither callback runs under the buffer or span-extension locks. Callbacks
/// must not recursively emit events into this same layer. They must not strongly
/// capture the buffer: a callback-owned buffer can reintroduce the subscriber /
/// report ownership cycle that the layer's weak writer avoids. Use independently
/// owned routing state or a weak application reference instead.
#[derive(Clone)]
pub struct CaptureLayer {
    buffer: WeakLogBuffer,
    active: Arc<dyn Fn() -> bool + Send + Sync>,
    inactive_writer: Arc<dyn Fn(&LogRecord) + Send + Sync>,
}

impl CaptureLayer {
    /// Route each event that passes this layer's filter.
    ///
    /// When `active` returns `true`, the layer queues the record in `buffer`.
    /// When it returns `false`, the layer calls `inactive_sink` immediately
    /// with the record. The caller chooses the sink, such as a stderr writer.
    /// A layer that always queues records can pass `Arc::new(|| true)` as its
    /// predicate and `|_| {}` as its sink.
    pub fn new(
        buffer: &LogBuffer,
        active: Arc<dyn Fn() -> bool + Send + Sync>,
        inactive_sink: impl Fn(&LogRecord) + Send + Sync + 'static,
    ) -> Self {
        Self {
            buffer: buffer.downgrade(),
            active,
            inactive_writer: Arc::new(inactive_sink),
        }
    }
}

impl fmt::Debug for CaptureLayer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Debugging a layer must not execute callbacks or render retained reports.
        formatter
            .debug_struct("CaptureLayer")
            .finish_non_exhaustive()
    }
}

impl<S> Layer<S> for CaptureLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        let mut fields = CapturedFields::for_span();
        attrs.record(&mut fields);
        // The independently filtered capture and human-output layers can share a
        // registry. Both observe the same span fields; do not insert twice.
        let mut extensions = span.extensions_mut();
        if extensions.get_mut::<CapturedFields>().is_none() {
            extensions.insert(fields);
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        let mut update = CapturedFields::for_span();
        values.record(&mut update);
        let replaced = {
            let mut extensions = span.extensions_mut();
            let Some(fields) = extensions.get_mut::<CapturedFields>() else {
                return;
            };
            fields.merge(&update)
        };
        // The final retained report can run an error destructor that emits a
        // log and reads this span's extensions again.
        drop(replaced);
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        let mut captured = CapturedFields::default();
        event.record(&mut captured);
        let spans = ctx
            .event_scope(event)
            .into_iter()
            .flat_map(|scope| scope.from_root())
            .map(|span| {
                let captured_fields = span
                    .extensions()
                    .get::<CapturedFields>()
                    .cloned()
                    .unwrap_or_default();
                let mut fields = captured_fields.fields;
                if let Some(message) = captured_fields.message {
                    fields.push(("message".to_string(), message.into()));
                }
                LogSpan {
                    name: Arc::from(span.name()),
                    fields,
                }
            })
            .collect();
        let metadata = event.metadata();
        let record = LogRecord {
            timestamp: SystemTime::now(),
            level: level(metadata.level()),
            target: Arc::from(metadata.target()),
            message: Arc::from(
                captured
                    .message
                    .unwrap_or_else(|| metadata.name().to_string()),
            ),
            fields: captured.fields,
            spans,
            file: metadata.file().map(Arc::from),
            line: metadata.line(),
        };
        if (self.active)() {
            self.buffer.push(record);
        } else {
            (self.inactive_writer)(&record);
        }
    }
}

fn level(level: &tracing::Level) -> LogLevel {
    match *level {
        tracing::Level::ERROR => LogLevel::Error,
        tracing::Level::WARN => LogLevel::Warn,
        tracing::Level::INFO => LogLevel::Info,
        tracing::Level::DEBUG => LogLevel::Debug,
        tracing::Level::TRACE => LogLevel::Trace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "eyre")]
    use crate::DiagnosticReport;
    #[cfg(feature = "eyre")]
    use crate::capture::LogValue;
    use crate::capture::format_record;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::thread;
    use tracing_subscriber::layer::SubscriberExt;
    fn active_probe(active: &Arc<AtomicBool>) -> Arc<dyn Fn() -> bool + Send + Sync> {
        let active = Arc::clone(active);
        Arc::new(move || active.load(Ordering::Acquire))
    }
    #[cfg(feature = "eyre")]
    #[test]
    fn span_report_fields_do_not_retain_the_span_captured_by_the_report() {
        let buffer = LogBuffer::new();
        let subscriber = tracing_subscriber::registry()
            .with(CaptureLayer::new(&buffer, Arc::new(|| true), |_| {}))
            .with(tracing_error::ErrorLayer::default());
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("report_owning_span", error = tracing::field::Empty);
            let _entered = span.enter();
            let report =
                DiagnosticReport::new(eyre::Report::new(tracing_error::TracedError::from(
                    std::io::Error::other("Synthetic span field failure"),
                )));
            let retained = Arc::downgrade(&report.report);
            span.record("error", report.as_error());
            drop(report);
            assert!(
                retained.upgrade().is_none(),
                "a span must not retain a report that can own that same span"
            );
            tracing::info!("Record with plain span diagnostics");
        });
        let records = buffer.records_since(&mut 0);
        assert_eq!(records.len(), 1);
        assert!(matches!(
            &records[0].spans[0].fields[0].1,
            LogValue::Text(text) if text.contains("Synthetic span field failure")
        ));
    }

    #[test]
    fn inactive_events_are_written_but_not_buffered() {
        let active = Arc::new(AtomicBool::new(false));
        let buffer = LogBuffer::new();
        let output = Arc::new(Mutex::new(Vec::<String>::new()));
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            active_probe(&active),
            {
                let output = Arc::clone(&output);
                move |record| {
                    output
                        .lock()
                        .expect("output lock")
                        .push(format_record(record))
                }
            },
        ));

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(source = "inactive", "not captured");
        });

        let mut cursor = 0;
        assert!(buffer.records_since(&mut cursor).is_empty());
        let output = output.lock().expect("output lock");
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("not captured"));
    }

    #[test]
    fn active_events_are_buffered_without_writing_to_the_inactive_sink() {
        let active = Arc::new(AtomicBool::new(true));
        let buffer = LogBuffer::new();
        let output = Arc::new(Mutex::new(Vec::<String>::new()));
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            active_probe(&active),
            {
                let output = Arc::clone(&output);
                move |record| {
                    output
                        .lock()
                        .expect("output lock")
                        .push(format_record(record))
                }
            },
        ));

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(source = "active", "captured");
        });

        let mut cursor = 0;
        let records = buffer.records_since(&mut cursor);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message.as_ref(), "captured");
        assert!(output.lock().expect("output lock").is_empty());
    }

    #[test]
    fn replay_preserves_later_presentation_of_structured_records() {
        let active = Arc::new(AtomicBool::new(true));
        let buffer = LogBuffer::new();
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            active_probe(&active),
            |_| {},
        ));

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("picker_handler", stage = "initial_load");
            let _entered = span.enter();
            tracing::warn!(query = "smith", "search returned no users");
        });

        active.store(false, Ordering::Release);
        let mut replay = Vec::new();
        buffer.replay_to(&mut replay).expect("replay should write");
        let replay = String::from_utf8(replay).expect("replay should be UTF-8");
        assert!(replay.contains("search returned no users"));
        assert!(replay.contains("query: smith"));
        assert!(replay.contains("picker_handler"));
        assert!(replay.contains("stage: initial_load"));
        assert!(replay.contains(" at "));
    }

    #[test]
    fn concurrent_event_producers_are_drained_without_losing_records() {
        let active = Arc::new(AtomicBool::new(true));
        let buffer = LogBuffer::new();
        let mut threads = Vec::new();
        for worker in 0..8 {
            let buffer = buffer.clone();
            let active = Arc::clone(&active);
            threads.push(thread::spawn(move || {
                let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
                    &buffer,
                    active_probe(&active),
                    |_| {},
                ));
                tracing::subscriber::with_default(subscriber, || {
                    for event in 0..32 {
                        tracing::info!(worker, event, "concurrent event");
                    }
                });
            }));
        }
        for thread in threads {
            thread.join().expect("event producer should finish");
        }

        let mut cursor = 0;
        let records = buffer.records_since(&mut cursor);
        assert_eq!(records.len(), 8 * 32);
        assert!(
            records
                .iter()
                .all(|record| record.message.as_ref() == "concurrent event")
        );
    }

    #[test]
    fn buffered_terminal_output_uses_the_same_formatter_as_immediate_output() {
        let active = Arc::new(AtomicBool::new(false));
        let buffer = LogBuffer::new();
        let immediate = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            active_probe(&active),
            {
                let immediate = Arc::clone(&immediate);
                move |record| immediate.lock().unwrap().push(format_record(record))
            },
        ));
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("drawing_update", path = "demo.excalidraw");
            let _entered = span.enter();
            for guarded in [false, true] {
                active.store(guarded, Ordering::Release);
                tracing::warn!(attempt = 3, "could not save drawing");
            }
        });
        let buffered = buffer.formatted_records_since(&mut 0);
        assert_eq!(buffered, *immediate.lock().unwrap());
        assert_eq!(buffered.len(), 1);
        assert!(buffered[0].contains("attempt: 3"));
        assert!(buffered[0].contains("drawing_update"));
        assert!(buffered[0].contains("demo.excalidraw"));
        assert!(buffered[0].contains("capture_layer.rs:"));
    }
    #[cfg(feature = "eyre")]
    #[test]
    fn event_reports_keep_the_typed_error_without_eager_formatting() {
        use std::sync::atomic::AtomicUsize;

        #[derive(Debug)]
        struct CountedError(Arc<AtomicUsize>);

        impl std::fmt::Display for CountedError {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fetch_add(1, Ordering::Relaxed);
                formatter.write_str("synthetic retained event error")
            }
        }
        impl std::error::Error for CountedError {}

        let buffer = LogBuffer::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            Arc::new(|| true),
            |_| {},
        ));
        tracing::subscriber::with_default(subscriber, || {
            let report = DiagnosticReport::new(eyre::Report::new(CountedError(Arc::clone(&calls))));
            let initial_calls = calls.load(Ordering::Relaxed);
            tracing::error!(error = report.as_error(), "Retained report event");
            let records = buffer.records_since(&mut 0);
            assert_eq!(calls.load(Ordering::Relaxed), initial_calls);
            let LogValue::Report(retained) = &records[0].fields[0].1 else {
                panic!("the event field must retain its report");
            };
            assert!(retained.report.downcast_ref::<CountedError>().is_some());
            let plain = crate::capture::format_record_plain(&records[0]);
            assert!(plain.contains("synthetic retained event error"));
            assert!(calls.load(Ordering::Relaxed) > initial_calls);
        });
    }

    #[test]
    fn multiple_layers_share_span_updates_without_duplicate_extensions() {
        let first = LogBuffer::new();
        let second = LogBuffer::new();
        let subscriber = tracing_subscriber::registry()
            .with(CaptureLayer::new(&first, Arc::new(|| true), |_| {}))
            .with(CaptureLayer::new(&second, Arc::new(|| true), |_| {}));
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("shared_context", phase = tracing::field::Empty);
            let _entered = span.enter();
            span.record("phase", "ready");
            tracing::info!("observed by both layers");
        });
        for buffer in [first, second] {
            let records = buffer.records_since(&mut 0);
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].spans[0].fields.len(), 1);
            assert_eq!(records[0].spans[0].fields[0].1.to_string(), "ready");
        }
    }
}
