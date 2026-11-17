//! Standalone synthetic preview; it does not listen or read Codex state.
use eframe::egui;
use std::sync::Arc;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::capture::CaptureLayer;
use teamy_tracing::capture::LogBuffer;
use teamy_tracing::gui::LogView;
use teamy_tracing::telemetry::Attribute;
use teamy_tracing::telemetry::RecordStore;
use teamy_tracing::telemetry::TelemetryRecord;
use teamy_tracing::telemetry::TelemetryValue;
use teamy_tracing::telemetry::view::RecordView;
use tracing_subscriber::prelude::*;

#[derive(Debug)]
struct Preview {
    store: RecordStore,
    view: RecordView,
    logs: LogBuffer,
    log_view: LogView,
}

impl eframe::App for Preview {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(root, |ui| {
            ui.heading("Synthetic telemetry preview");
            ui.label("Invented example data; no network collector or service is running.");
            egui::CollapsingHeader::new("Serialized telemetry").show(ui, |ui| {
                self.view.show(ui, &self.store);
            });
            ui.heading("Live report-aware capture");
            self.log_view.show(ui, &self.logs);
        });
    }
}

fn sample(event: &str, result: bool) -> TelemetryRecord {
    let text = |key: &str, value: &str| Attribute {
        key: key.into(),
        value: TelemetryValue::String(value.into()),
    };
    let mut attributes = vec![
        text("conversation.id", "example-session"),
        text("call_id", "example-call"),
    ];
    if result {
        attributes.push(text(
            "arguments",
            r#"{"cmd":"Get-Date","shell":"powershell"}"#,
        ));
        attributes.push(text(
            "output",
            "Monday, January 1, 2024\n[... telemetry preview truncated ...]",
        ));
        attributes.push(Attribute {
            key: "output_truncated".into(),
            value: TelemetryValue::Bool(true),
        });
    } else {
        attributes.push(text("decision", "approved"));
    }
    TelemetryRecord {
        event_name: event.into(),
        resource_attributes: vec![text("service.instance.id", "example-producer")],
        attributes,
        ..TelemetryRecord::default()
    }
}

fn main() -> eframe::Result<()> {
    let mut store = RecordStore::default();
    let _ = store.push(sample("codex.tool_decision", false));
    let _ = store.push(sample("codex.tool_result", true));
    // This preview produces only these fixed synthetic records. A long-lived
    // application should choose LogBuffer::bounded for its viewer history.
    let logs = LogBuffer::new();
    let subscriber =
        tracing_subscriber::registry().with(CaptureLayer::new(&logs, Arc::new(|| true), |_| {}));
    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!("synthetic_operation", item = "example");
        let _entered = span.enter();
        tracing::info!(attempt = 1, "Started invented operation");
        tracing::warn!("Synthetic warning for the log viewer");
        let report = DiagnosticReport::new(
            eyre::eyre!("Synthetic source error").wrap_err("Invented operation failed"),
        );
        tracing::error!(error = report.as_error(), "Retained synthetic diagnostic");
    });
    eframe::run_native(
        "Synthetic telemetry preview",
        eframe::NativeOptions::default(),
        Box::new(move |_| {
            Ok(Box::new(Preview {
                store,
                view: RecordView::default(),
                logs,
                log_view: LogView::default(),
            }))
        }),
    )
}
