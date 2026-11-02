//! Standalone synthetic preview; it does not listen or read Codex state.
use teamy_tracing::telemetry::Attribute;
use teamy_tracing::telemetry::RecordStore;
use teamy_tracing::telemetry::TelemetryRecord;
use teamy_tracing::telemetry::TelemetryValue;
use teamy_tracing::telemetry::view::RecordView;

#[derive(Debug)]
struct Preview {
    store: RecordStore,
    view: RecordView,
}

impl eframe::App for Preview {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(context, |ui| {
            ui.heading("Synthetic telemetry preview");
            ui.label("Invented example data; no collector is running.");
            self.view.show(ui, &self.store);
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
    eframe::run_native(
        "Synthetic telemetry preview",
        eframe::NativeOptions::default(),
        Box::new(move |_| {
            Ok(Box::new(Preview {
                store,
                view: RecordView::default(),
            }))
        }),
    )
}
