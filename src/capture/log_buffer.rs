// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use super::LogRecord;
use super::format_record;
use super::log_buffer_state::LogBufferState;
use super::weak_log_buffer::WeakLogBuffer;
use std::fmt;
use std::io::Write;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

/// Application-owned live records with absolute read cursors.
///
/// Clones share one store. `new` retains every record until acknowledged, which
/// suits output still owed to a terminal. `bounded` evicts oldest records by
/// count; retained reports have unknown size, so this is not a byte-memory limit.
/// Repaint scheduling, output routing and subscriber installation belong to the
/// application. No user formatting, sink, or final report destructor runs under
/// the store lock.
#[derive(Clone)]
pub struct LogBuffer {
    inner: Arc<Mutex<LogBufferState>>,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self::from_state(Arc::new(Mutex::new(LogBufferState {
            records: Default::default(),
            first_index: 0,
            capacity: None,
        })))
    }

    /// Retain at most `capacity` records, evicting oldest records on insertion.
    pub fn bounded(capacity: NonZeroUsize) -> Self {
        Self::from_state(Arc::new(Mutex::new(LogBufferState {
            records: Default::default(),
            first_index: 0,
            capacity: Some(capacity.get()),
        })))
    }

    pub(super) fn from_state(inner: Arc<Mutex<LogBufferState>>) -> Self {
        Self { inner }
    }

    pub(super) fn downgrade(&self) -> WeakLogBuffer {
        WeakLogBuffer::new(Arc::downgrade(&self.inner))
    }

    /// Snapshot retained records since the absolute `cursor`, advancing it to
    /// the current end. A cursor older than eviction/acknowledgement starts at
    /// the oldest surviving record. Cloned reports keep their original handler.
    pub fn records_since(&self, cursor: &mut usize) -> Vec<LogRecord> {
        let state = self.lock_state();
        let start = cursor
            .saturating_sub(state.first_index)
            .min(state.records.len());
        *cursor = state.first_index + state.records.len();
        state.records.iter().skip(start).cloned().collect()
    }

    /// Format a snapshot using the same terminal policy as immediate output.
    pub fn formatted_records_since(&self, cursor: &mut usize) -> Vec<String> {
        self.records_since(cursor)
            .iter()
            .map(format_record)
            .collect()
    }

    /// Release successfully consumed records through a read cursor. Events that
    /// arrive after that read remain retained; repeated acknowledgements are safe.
    pub fn acknowledge_through(&self, cursor: usize) {
        let removed = {
            let mut state = self.lock_state();
            let count = cursor
                .saturating_sub(state.first_index)
                .min(state.records.len());
            let removed: Vec<_> = state.records.drain(..count).collect();
            state.first_index += count;
            removed
        };
        drop(removed);
    }

    /// Replay a snapshot to the caller's writer. Does not acknowledge records;
    /// the caller decides whether the whole delivery succeeded.
    pub fn replay_to<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        for record in self.records_since(&mut 0) {
            writeln!(writer, "{}", format_record(&record))?;
        }
        Ok(())
    }

    /// Append a record, releasing any evicted report after unlocking the store.
    pub fn push(&self, record: LogRecord) {
        let removed = {
            let mut state = self.lock_state();
            state.records.push_back(record);
            if state
                .capacity
                .is_some_and(|capacity| state.records.len() > capacity)
            {
                state.first_index += 1;
                state.records.pop_front()
            } else {
                None
            }
        };
        drop(removed);
    }

    fn lock_state(&self) -> MutexGuard<'_, LogBufferState> {
        // Mutations invoke no user code, so poisoning cannot leave partial
        // ownership changes. Preserve already retained diagnostics on recovery.
        match self.inner.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for LogBuffer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Neither a retained report's formatter nor an output callback is run.
        let (retained_records, first_index, capacity) = {
            let state = self.lock_state();
            (state.records.len(), state.first_index, state.capacity)
        };
        formatter
            .debug_struct("LogBuffer")
            .field("retained_records", &retained_records)
            .field("first_index", &first_index)
            .field("capacity", &capacity)
            .finish()
    }
}

#[cfg(all(test, feature = "eyre"))]
mod tests {
    use super::*;
    use crate::DiagnosticReport;
    use crate::capture::CaptureLayer;
    use crate::capture::LogLevel;
    use crate::capture::LogValue;
    use std::fmt;
    use tracing_subscriber::layer::SubscriberExt;

    #[derive(Debug)]
    struct LoggingDropError;

    impl fmt::Display for LoggingDropError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("Synthetic retained error")
        }
    }

    impl std::error::Error for LoggingDropError {}

    impl Drop for LoggingDropError {
        fn drop(&mut self) {
            tracing::warn!("Emitted while dropping a retained error");
        }
    }

    fn report_record() -> LogRecord {
        LogRecord {
            timestamp: std::time::SystemTime::UNIX_EPOCH,
            level: LogLevel::Error,
            target: Arc::from("test"),
            message: Arc::from("Retained error"),
            fields: vec![(
                "error".to_owned(),
                LogValue::Report(DiagnosticReport::new(eyre::Report::new(LoggingDropError))),
            )],
            spans: Vec::new(),
            file: None,
            line: None,
        }
    }

    #[test]
    fn dropping_the_buffer_releases_reports_that_retain_their_subscriber() {
        let buffer = LogBuffer::new();
        let subscriber = tracing_subscriber::registry()
            .with(CaptureLayer::new(&buffer, Arc::new(|| true), |_| {}))
            .with(tracing_error::ErrorLayer::default());
        let dispatch = tracing::Dispatch::new(subscriber);
        let retained_subscriber = dispatch.downgrade();
        let retained_report = tracing::dispatcher::with_default(&dispatch, || {
            let span = tracing::info_span!("retained_report_context");
            let _entered = span.enter();
            let report =
                DiagnosticReport::new(eyre::Report::new(tracing_error::TracedError::from(
                    std::io::Error::other("Synthetic captured failure"),
                )));
            tracing::error!(error = report.as_error(), "Buffered diagnostic");
            Arc::downgrade(&report.report)
        });
        drop(dispatch);
        assert!(retained_subscriber.upgrade().is_some());
        let records = buffer.records_since(&mut 0);
        assert_eq!(records.len(), 1);
        drop(buffer);
        assert!(
            retained_report.upgrade().is_some(),
            "a cloned record must still own its shared diagnostic"
        );
        drop(records);
        assert!(retained_report.upgrade().is_none());
        assert!(retained_subscriber.upgrade().is_none());
    }

    #[test]
    fn acknowledging_a_report_allows_its_destructor_to_emit_logs() {
        let buffer = LogBuffer::new();
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            Arc::new(|| true),
            |_| {},
        ));
        tracing::subscriber::with_default(subscriber, || {
            buffer.push(report_record());
            let mut cursor = 0;
            assert_eq!(buffer.records_since(&mut cursor).len(), 1);
            buffer.acknowledge_through(cursor);
            let records = buffer.records_since(&mut cursor);
            assert_eq!(records.len(), 1);
            assert_eq!(
                records[0].message.as_ref(),
                "Emitted while dropping a retained error"
            );
        });
    }

    #[test]
    fn evicting_a_report_allows_its_destructor_to_emit_logs() {
        let buffer = LogBuffer::bounded(NonZeroUsize::new(1).unwrap());
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            Arc::new(|| true),
            |_| {},
        ));
        tracing::subscriber::with_default(subscriber, || {
            buffer.push(report_record());
            // Push directly to exercise destruction outside the dispatcher,
            // whose scoped reentrancy guard can suppress nested log events.
            buffer.push(LogRecord {
                timestamp: std::time::SystemTime::UNIX_EPOCH,
                level: LogLevel::Info,
                target: Arc::from("test"),
                message: Arc::from("Eviction trigger"),
                fields: Vec::new(),
                spans: Vec::new(),
                file: None,
                line: None,
            });
            let records = buffer.records_since(&mut 0);
            assert_eq!(records.len(), 1);
            assert_eq!(
                records[0].message.as_ref(),
                "Emitted while dropping a retained error"
            );
        });
    }

    #[test]
    fn report_rendering_can_emit_logs_during_replay() {
        #[derive(Debug)]
        struct LoggingError;

        impl fmt::Display for LoggingError {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                tracing::warn!("Emitted while rendering a retained error");
                formatter.write_str("Synthetic error detail")
            }
        }

        impl std::error::Error for LoggingError {}

        let buffer = LogBuffer::new();
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            Arc::new(|| true),
            |_| {},
        ));
        tracing::subscriber::with_default(subscriber, || {
            let report = DiagnosticReport::new(eyre::Report::new(LoggingError));
            tracing::error!(error = report.as_error(), "Retained error");
            let mut output = Vec::new();
            buffer.replay_to(&mut output).unwrap();
            assert!(
                String::from_utf8(output)
                    .unwrap()
                    .contains("Synthetic error detail")
            );
            assert!(buffer.records_since(&mut 0).iter().any(|record| {
                record.message.as_ref() == "Emitted while rendering a retained error"
            }));
        });
    }

    #[test]
    fn a_panicking_replay_writer_does_not_discard_or_break_buffered_logs() {
        struct PanickingWriter;

        impl Write for PanickingWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                panic!("synthetic replay writer panic");
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let buffer = LogBuffer::new();
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            Arc::new(|| true),
            |_| {},
        ));
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("retained before replay");
            assert!(std::panic::catch_unwind(|| buffer.replay_to(PanickingWriter)).is_err());
            tracing::info!("retained after replay");

            let mut cursor = 0;
            let records = buffer.records_since(&mut cursor);
            let messages: Vec<_> = records
                .iter()
                .map(|record| record.message.as_ref())
                .collect();
            assert_eq!(
                messages,
                ["retained before replay", "retained after replay"]
            );
            buffer.acknowledge_through(cursor);
            assert!(buffer.records_since(&mut cursor).is_empty());
        });
    }

    #[test]
    fn acknowledging_printed_logs_keeps_events_arriving_during_output() {
        let buffer = LogBuffer::new();
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            Arc::new(|| true),
            |_| {},
        ));
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("printed");
            let mut cursor = 0;
            let formatted = buffer.formatted_records_since(&mut cursor);
            assert_eq!(formatted.len(), 1);
            assert!(formatted[0].contains("printed"));
            assert!(formatted[0].contains("log_buffer.rs:"));
            tracing::info!("arrived while printing");
            buffer.acknowledge_through(cursor);
            // A stale acknowledgement must not delete more recent events.
            buffer.acknowledge_through(cursor);
            let remaining = buffer.records_since(&mut cursor);
            assert_eq!(remaining.len(), 1);
            assert_eq!(remaining[0].message.as_ref(), "arrived while printing");
            let mut replay = Vec::new();
            buffer.replay_to(&mut replay).unwrap();
            let replay = String::from_utf8(replay).unwrap();
            assert!(!replay.contains("printed"));
            assert!(replay.contains("arrived while printing"));
            buffer.acknowledge_through(cursor);
            assert!(buffer.records_since(&mut 0).is_empty());
        });
    }
}

#[cfg(test)]
mod cursor_tests {
    use super::*;
    use crate::capture::CaptureLayer;
    use crate::capture::LogLevel;
    use std::fmt::Write as _;
    use std::time::SystemTime;
    use tracing_subscriber::layer::SubscriberExt;

    fn record(message: &str) -> LogRecord {
        LogRecord {
            timestamp: SystemTime::UNIX_EPOCH,
            level: LogLevel::Info,
            target: Arc::from("synthetic"),
            message: Arc::from(message),
            fields: Vec::new(),
            spans: Vec::new(),
            file: None,
            line: None,
        }
    }

    #[test]
    fn an_evicted_cursor_resumes_at_the_oldest_record_and_ack_keeps_newer_events() {
        let buffer = LogBuffer::bounded(NonZeroUsize::new(2).unwrap());
        for message in ["evicted", "consumed", "not consumed"] {
            buffer.push(record(message));
        }
        let mut cursor = 0;
        let initial = buffer.records_since(&mut cursor);
        assert_eq!(cursor, 3);
        assert_eq!(
            initial
                .iter()
                .map(|record| record.message.as_ref())
                .collect::<Vec<_>>(),
            ["consumed", "not consumed"]
        );

        buffer.acknowledge_through(2);
        buffer.push(record("arrived later"));
        let mut stale = 0;
        let remaining = buffer.records_since(&mut stale);
        assert_eq!(stale, 4);
        assert_eq!(
            remaining
                .iter()
                .map(|record| record.message.as_ref())
                .collect::<Vec<_>>(),
            ["not consumed", "arrived later"]
        );
    }

    #[test]
    fn debug_formatting_does_not_hold_the_capture_lock() {
        struct LoggingWriter;
        impl fmt::Write for LoggingWriter {
            fn write_str(&mut self, _: &str) -> fmt::Result {
                tracing::info!("emitted by debug's output writer");
                Ok(())
            }
        }

        let buffer = LogBuffer::new();
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            Arc::new(|| true),
            |_| {},
        ));
        tracing::subscriber::with_default(subscriber, || {
            write!(LoggingWriter, "{buffer:?}").unwrap();
        });
        assert!(!buffer.records_since(&mut 0).is_empty());
    }
}
