#[cfg(feature = "eyre")]
use super::DiagnosticReportView;
use crate::capture::LogBuffer;
use crate::capture::LogLevel;
use crate::capture::LogRecord;
use crate::capture::LogValue;
use egui::Color32;
use egui::ComboBox;
use egui::RichText;
use egui::ScrollArea;
use egui::Ui;

/// Filterable view of an application-owned live log buffer.
///
/// Each frame takes a snapshot of the buffer's retained records. Bounded eviction
/// and explicit acknowledgement are reflected in the next snapshot. Captured
/// reports remain shared with their original handlers and render on expansion.
/// Applications own repaint scheduling when records arrive.
#[derive(Debug)]
pub struct LogView {
    /// Case-insensitive text matched against messages, source, field names,
    /// text field values, and spans. Report bodies render when expanded.
    pub filter: String,
    /// Lowest severity displayed; this does not change the capture layer filter.
    pub minimum_level: LogLevel,
    /// Keep the scroll position at the newest entry until manually scrolled up.
    pub follow_tail: bool,
}

impl Default for LogView {
    fn default() -> Self {
        Self {
            filter: String::new(),
            minimum_level: LogLevel::Trace,
            follow_tail: true,
        }
    }
}

impl LogView {
    /// Render retained logs without acknowledging or removing them.
    pub fn show(&mut self, ui: &mut Ui, buffer: &LogBuffer) {
        ui.horizontal(|ui| {
            ui.label("Filter");
            ui.text_edit_singleline(&mut self.filter);
            ComboBox::from_id_salt("minimum_log_level")
                .selected_text(format!("{:?}", self.minimum_level))
                .show_ui(ui, |ui| {
                    for level in [
                        LogLevel::Trace,
                        LogLevel::Debug,
                        LogLevel::Info,
                        LogLevel::Warn,
                        LogLevel::Error,
                    ] {
                        ui.selectable_value(&mut self.minimum_level, level, format!("{level:?}"));
                    }
                });
            ui.checkbox(&mut self.follow_tail, "Follow latest");
        });
        // Taking a new snapshot each frame keeps one retention authority. Its
        // end cursor supplies stable identities even after bounded eviction.
        let mut end = 0;
        let records = buffer.records_since(&mut end);
        let first = end.saturating_sub(records.len());
        let filter = self.filter.to_lowercase();
        let mut visible = 0;
        if first > 0 {
            ui.weak(format!("{first} earlier records are no longer retained"));
        }
        ScrollArea::vertical()
            .stick_to_bottom(self.follow_tail)
            .show(ui, |ui| {
                for (index, record) in records.iter().enumerate() {
                    if !self.matches_normalized(record, &filter) {
                        continue;
                    }
                    visible += 1;
                    egui::CollapsingHeader::new(
                        RichText::new(format!(
                            "{:?} · {}",
                            record.level,
                            super::escape_display(&record.message)
                        ))
                        .color(Self::level_color(record.level)),
                    )
                    .id_salt(("live_log", first + index))
                    .show(ui, |ui| Self::show_record(ui, record));
                }
            });
        ui.label(format!("{visible} shown · {} retained", records.len()));
    }

    /// Test the current text and severity filter without rendering a widget.
    pub fn matches(&self, record: &LogRecord) -> bool {
        self.matches_normalized(record, &self.filter.to_lowercase())
    }

    fn matches_normalized(&self, record: &LogRecord, filter: &str) -> bool {
        if Self::severity(record.level) < Self::severity(self.minimum_level) {
            return false;
        }
        if filter.is_empty() {
            return true;
        }
        record.message.to_lowercase().contains(filter)
            || record.target.to_lowercase().contains(filter)
            || record
                .file
                .as_ref()
                .is_some_and(|file| file.to_lowercase().contains(filter))
            || record
                .fields
                .iter()
                .any(|(name, value)| Self::field_matches_filter(name, value, filter))
            || record.spans.iter().any(|span| {
                span.name.to_lowercase().contains(filter)
                    || span
                        .fields
                        .iter()
                        .any(|(name, value)| Self::field_matches_filter(name, value, filter))
            })
    }

    fn field_matches_filter(name: &str, value: &LogValue, filter: &str) -> bool {
        name.to_lowercase().contains(filter)
            || match value {
                LogValue::Text(text) => text.to_lowercase().contains(filter),
                #[cfg(feature = "eyre")]
                LogValue::Report(_) => false,
            }
    }

    fn show_record(ui: &mut Ui, record: &LogRecord) {
        ui.monospace(super::escape_display(&record.target));
        ui.label(format!("Captured: {:?}", record.timestamp));
        if let Some(file) = &record.file {
            let mut location = super::escape_display(file);
            if let Some(line) = record.line {
                location.push(':');
                location.push_str(&line.to_string());
            }
            ui.monospace(location);
        }
        for (name, value) in &record.fields {
            Self::show_field(ui, name, value);
        }
        for (index, span) in record.spans.iter().enumerate() {
            egui::CollapsingHeader::new(format!("in {}", super::escape_display(&span.name)))
                .id_salt(("span", index))
                .show(ui, |ui| {
                    for (name, value) in &span.fields {
                        Self::show_field(ui, name, value);
                    }
                });
        }
    }

    fn show_field(ui: &mut Ui, name: &str, value: &LogValue) {
        #[cfg(feature = "eyre")]
        if let LogValue::Report(report) = value {
            egui::CollapsingHeader::new(format!("{}: error details", super::escape_display(name)))
                .id_salt(name)
                .show(ui, |ui| DiagnosticReportView::new(report).show(ui));
            return;
        }
        ui.monospace(format!(
            "{}: {}",
            super::escape_display(name),
            super::escape_display(&value.to_string())
        ));
    }

    fn severity(level: LogLevel) -> u8 {
        match level {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
        }
    }

    fn level_color(level: LogLevel) -> Color32 {
        match level {
            LogLevel::Trace | LogLevel::Debug => Color32::GRAY,
            LogLevel::Info => Color32::from_rgb(0, 120, 190),
            LogLevel::Warn => Color32::from_rgb(180, 110, 0),
            LogLevel::Error => Color32::from_rgb(210, 40, 40),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::CaptureLayer;
    use crate::capture::LogSpan;
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::time::SystemTime;
    use tracing_subscriber::prelude::*;

    #[test]
    fn filter_searches_context_and_fields_without_losing_severity_selection() {
        let record = LogRecord {
            timestamp: SystemTime::UNIX_EPOCH,
            level: LogLevel::Warn,
            target: Arc::from("worker"),
            message: Arc::from("operation failed"),
            fields: vec![("attempt".into(), LogValue::Text("third".into()))],
            spans: vec![LogSpan {
                name: Arc::from("synchronization"),
                fields: vec![("document".into(), LogValue::Text("draft.txt".into()))],
            }],
            file: Some(Arc::from("background.rs")),
            line: Some(5),
        };
        let mut view = LogView::default();
        for filter in [
            "WORKER",
            "third",
            "Synchronization",
            "DRAFT.TXT",
            "background.rs",
        ] {
            view.filter = filter.into();
            assert!(view.matches(&record), "{filter}");
        }
        view.minimum_level = LogLevel::Error;
        assert!(!view.matches(&record));
        view.minimum_level = LogLevel::Trace;
        view.filter = "missing".into();
        assert!(!view.matches(&record));
    }

    #[cfg(feature = "eyre")]
    #[test]
    fn filtering_matches_report_field_names_without_rendering_reports() {
        use crate::DiagnosticReport;
        use std::fmt;
        use std::sync::atomic::AtomicUsize;
        use std::sync::atomic::Ordering;

        #[derive(Debug)]
        struct CountedError(Arc<AtomicUsize>);

        impl fmt::Display for CountedError {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fetch_add(1, Ordering::Relaxed);
                formatter.write_str("synthetic report body")
            }
        }

        impl std::error::Error for CountedError {}

        let formatted = Arc::new(AtomicUsize::new(0));
        let report = DiagnosticReport::new(eyre::Report::new(CountedError(Arc::clone(&formatted))));
        let before_filter = formatted.load(Ordering::Relaxed);
        let record = LogRecord {
            timestamp: SystemTime::UNIX_EPOCH,
            level: LogLevel::Error,
            target: Arc::from("worker"),
            message: Arc::from("operation failed"),
            fields: vec![("diagnostic".into(), LogValue::Report(report))],
            spans: Vec::new(),
            file: None,
            line: None,
        };
        let mut view = LogView {
            filter: "synthetic report body".into(),
            ..Default::default()
        };

        assert!(!view.matches(&record));
        view.filter = "DIAGNOSTIC".into();
        assert!(view.matches(&record));
        assert_eq!(formatted.load(Ordering::Relaxed), before_filter);
    }

    #[test]
    fn a_view_uses_the_buffers_current_bounded_history_without_acknowledgement() {
        let buffer = LogBuffer::bounded(NonZeroUsize::new(2).unwrap());
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(
            &buffer,
            Arc::new(|| true),
            |_| {},
        ));
        let mut view = LogView::default();
        let context = egui::Context::default();
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("first");
            tracing::warn!("second");
            tracing::error!("third");
            let mut output = context.run_ui(egui::RawInput::default(), |ui| view.show(ui, &buffer));
            // This headless test has no renderer to apply texture uploads.
            output.textures_delta.clear();
            assert!(!output.shapes.is_empty());
            tracing::debug!("fourth");
            let mut output = context.run_ui(egui::RawInput::default(), |ui| view.show(ui, &buffer));
            output.textures_delta.clear();
        });
        let mut cursor = 0;
        let retained = buffer.records_since(&mut cursor);
        assert_eq!(cursor, 4);
        assert_eq!(retained.len(), 2);
        assert_eq!(retained[0].message.as_ref(), "third");
        assert_eq!(retained[1].message.as_ref(), "fourth");
    }
}
